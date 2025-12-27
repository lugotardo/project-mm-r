mod auth;

use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::broadcast;
use common::*;
use world::World;
use auth::{AuthState, SharedAuthState};
use ipc::{EventHub, GameEvent, SharedEventHub};

pub type SharedGameState = Arc<Mutex<GameState>>;

pub struct GameState {
    pub world: World,
    pub players: HashMap<uuid::Uuid, PlayerSession>,
    pub tick_update_tx: broadcast::Sender<GameUpdate>,
}

pub struct PlayerSession {
    pub user_id: uuid::Uuid,
    pub entity_id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameUpdate {
    pub tick: u64,
    pub viewport: ViewportData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ViewportData {
    pub tiles: Vec<TileData>,
    pub entities: Vec<EntityData>,
    pub player_pos: Position,
    pub width: i32,
    pub height: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TileData {
    pub x: i32,
    pub y: i32,
    pub glyph: char,
    pub fg_color: String,
    pub bg_color: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntityData {
    pub x: i32,
    pub y: i32,
    pub glyph: char,
    pub color: String,
    pub name: String,
}

impl GameState {
    pub fn new() -> Self {
        let (tick_tx, _) = broadcast::channel(100);
        Self {
            world: World::new(50, 50),
            players: HashMap::new(),
            tick_update_tx: tick_tx,
        }
    }

    pub fn spawn_player(&mut self, user_id: uuid::Uuid, name: String) -> Option<u32> {
        if let Some(entity_id) = self.world.spawn_entity(
            name.clone(),
            Position::new(10, 10),
            EntityType::Player,
        ) {
            self.players.insert(user_id, PlayerSession {
                user_id,
                entity_id,
                name,
            });
            Some(entity_id)
        } else {
            None
        }
    }

    pub fn get_viewport(&self, entity_id: u32, view_range: i32) -> Option<ViewportData> {
        let entity = self.world.get_entity(entity_id)?;
        let center = entity.pos;

        let mut tiles = Vec::new();
        let mut entities_data = Vec::new();

        for y in (center.y - view_range)..=(center.y + view_range) {
            for x in (center.x - view_range)..=(center.x + view_range) {
                let pos = Position::new(x, y);
                if let Some(tile) = self.world.get_tile(pos) {
                    let (glyph, fg, bg) = tile_to_glyph(&tile);
                    tiles.push(TileData { x, y, glyph, fg_color: fg, bg_color: bg });
                }
            }
        }

        let visible_entities = self.world.get_entities_in_region(center, view_range);
        for entity in visible_entities {
            let (glyph, color) = entity_to_glyph(entity);
            entities_data.push(EntityData {
                x: entity.pos.x,
                y: entity.pos.y,
                glyph,
                color,
                name: entity.name.clone(),
            });
        }

        Some(ViewportData {
            tiles,
            entities: entities_data,
            player_pos: center,
            width: view_range * 2 + 1,
            height: view_range * 2 + 1,
        })
    }
}

const DEBUG_MODE: bool = true; // ← Modo debug

fn tile_to_glyph(tile: &Tile) -> (char, String, String) {
    match tile.terrain {
        TerrainType::Grass => ('░', "#4a4".to_string(), "#232".to_string()),
        TerrainType::Water => ('≈', "#24a".to_string(), "#012".to_string()),
        TerrainType::Stone => ('█', "#888".to_string(), "#444".to_string()),
        TerrainType::Sand => ('·', "#dc6".to_string(), "#a94".to_string()),
    }
}

fn entity_to_glyph(entity: &Entity) -> (char, String) {
    match entity.entity_type {
        EntityType::Player => ('@', "#ff0".to_string()),
        EntityType::NPC => ('H', "#0af".to_string()),
        EntityType::Animal => ('d', "#fa0".to_string()),
    }
}

#[tokio::main]
async fn main() {
    println!("🎮 MM Game Server Starting...");

    let auth_state: SharedAuthState = Arc::new(Mutex::new(AuthState::new()));
    let game_state: SharedGameState = Arc::new(Mutex::new(GameState::new()));
    let event_hub: SharedEventHub = Arc::new(Mutex::new(EventHub::new()));

    let auth_filter = warp::any().map(move || auth_state.clone());
    let game_filter = warp::any().map(move || game_state.clone());
    let event_filter = warp::any().map(move || event_hub.clone());

    // === CORS Configuration ===
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_headers(vec!["Content-Type", "Authorization"]);

    // Rota: POST /auth/register
    let register_route = warp::path!("auth" / "register")
        .and(warp::post())
        .and(warp::body::json())
        .and(auth_filter.clone())
        .map(|req: LoginRequest, auth: SharedAuthState| {
            let mut auth = auth.lock().unwrap();
            match auth.register_user(req.username, req.password) {
                Ok(_) => warp::reply::json(&LoginResponse {
                    success: true,
                    token: None,
                    message: "Usuário criado com sucesso!".to_string(),
                    role: None,
                }),
                Err(msg) => warp::reply::json(&LoginResponse {
                    success: false,
                    token: None,
                    message: msg,
                    role: None,
                }),
            }
        });

    // Rota: POST /auth/login
    let login_route = warp::path!("auth" / "login")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::addr::remote())
        .and(auth_filter.clone())
        .map(|req: LoginRequest, addr: Option<std::net::SocketAddr>, auth: SharedAuthState| {
            let ip = addr.map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
            let mut auth = auth.lock().unwrap();
            let response = auth.login(req.username, req.password, ip);
            warp::reply::json(&response)
        });

    // WebSocket autenticado
    let ws_game = warp::path!("ws" / "game")
        .and(warp::ws())
        .and(auth_filter.clone())
        .and(game_filter.clone())
        .and(event_filter.clone())
        .map(|ws: warp::ws::Ws, auth: SharedAuthState, game: SharedGameState, events: SharedEventHub| {
            ws.on_upgrade(move |socket| handle_game_websocket(socket, auth, game, events))
        });

    // Endpoint para admin-server se conectar
    let event_stream = warp::path!("api" / "events" / "stream")
        .and(warp::ws())
        .and(event_filter.clone())
        .map(|ws: warp::ws::Ws, events: SharedEventHub| {
            ws.on_upgrade(move |socket| stream_events_to_admin(socket, events))
        });

    // Arquivos estáticos
    let static_files = warp::fs::dir("./web-client");

    // === API REST PARA ADMIN ===
    
    // GET /api/world/map - Retorna mapa completo
    let api_world_map = warp::path!("api" / "world" / "map")
        .and(warp::get())
        .and(game_filter.clone())
        .map(|game: SharedGameState| {
            let game = game.lock().unwrap();
            let (width, height) = game.world.dimensions();
            
            let mut tiles = Vec::new();
            for y in 0..height {
                for x in 0..width {
                    let pos = Position::new(x, y);
                    if let Some(tile) = game.world.get_tile(pos) {
                        let (glyph, fg, bg) = tile_to_glyph(&tile);
                        tiles.push(TileData {
                            x, y, glyph,
                            fg_color: fg,
                            bg_color: bg,
                        });
                    }
                }
            }
            
            warp::reply::json(&serde_json::json!({
                "width": width,
                "height": height,
                "tiles": tiles
            }))
        });
    
    // GET /api/players - Retorna lista de jogadores online
    let api_players = warp::path!("api" / "players")
        .and(warp::get())
        .and(game_filter.clone())
        .map(|game: SharedGameState| {
            let game = game.lock().unwrap();
            
            let players: Vec<_> = game.players.iter().map(|(user_id, session)| {
                let entity = game.world.get_entity(session.entity_id);
                serde_json::json!({
                    "id": user_id.to_string(),
                    "name": session.name,
                    "entity_id": session.entity_id,
                    "position": entity.map(|e| serde_json::json!({
                        "x": e.pos.x,
                        "y": e.pos.y
                    }))
                })
            }).collect();
            
            warp::reply::json(&players)
        });
    
    // GET /api/entities - Retorna todas as entidades
    let api_entities = warp::path!("api" / "entities")
        .and(warp::get())
        .and(game_filter.clone())
        .map(|game: SharedGameState| {
            let game = game.lock().unwrap();
            
            let entities: Vec<_> = game.world.get_all_entities()
                .iter()
                .map(|entity| {
                    serde_json::json!({
                        "id": entity.id,
                        "name": entity.name,
                        "type": format!("{:?}", entity.entity_type),
                        "position": {
                            "x": entity.pos.x,
                            "y": entity.pos.y
                        }
                    })
                })
                .collect();
            
            warp::reply::json(&entities)
        });

    let routes = register_route
        .or(login_route)
        .or(api_world_map)
        .or(api_players)
        .or(api_entities)
        .or(ws_game)
        .or(event_stream)
        .or(static_files)
        .with(cors);  // ← Adiciona CORS a todas as rotas

    println!("🎮 Game Server: http://127.0.0.1:8080");
    println!("📡 API endpoints:");
    println!("   GET /api/world/map");
    println!("   GET /api/players");
    println!("   GET /api/entities");
    println!("✅ CORS habilitado para todas as origens");
    
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_game_websocket(
    ws: warp::ws::WebSocket,
    _auth_state: SharedAuthState,  // Prefixado com _ para indicar que não é usado em DEBUG_MODE
    game_state: SharedGameState,
    event_hub: SharedEventHub,
) {
    use futures::{StreamExt, SinkExt};

    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut user_id: Option<uuid::Uuid> = None;
    let mut entity_id: Option<u32> = None;
    let mut player_name: String = String::new();

    while let Some(result) = ws_rx.next().await {
        if let Ok(msg) = result {
            if let Ok(text) = msg.to_str() {
                
                // LOGIN
                if DEBUG_MODE && user_id.is_none() {
                    if let Ok(simple_msg) = serde_json::from_str::<ClientMessage>(text) {
                        if let ClientMessage::Login { player_name: name } = simple_msg {
                            println!("🔧 DEBUG: Login sem autenticação: {}", name);
                            
                            let uid = uuid::Uuid::new_v4();
                            user_id = Some(uid);
                            player_name = name.clone();
                            
                            let response = {
                                let mut game = game_state.lock().unwrap();
                                if let Some(eid) = game.spawn_player(uid, name.clone()) {
                                    entity_id = Some(eid);
                                    
                                    // 🔔 BROADCAST EVENTO
                                    let pos = game.world.get_entity(eid)
                                        .map(|e| e.pos)
                                        .unwrap_or(Position::new(0, 0));
                                    
                                    event_hub.lock().unwrap().broadcast(GameEvent::PlayerConnected {
                                        name: name.clone(),
                                        id: uid.to_string(),
                                    });
                                    
                                    event_hub.lock().unwrap().broadcast(GameEvent::PlayerSpawned {
                                        name: name,
                                        pos,
                                    });
                                    
                                    game.get_viewport(eid, 15).map(|viewport| {
                                        GameUpdate {
                                            tick: game.world.get_current_tick(),
                                            viewport,
                                        }
                                    })
                                } else {
                                    None
                                }
                            };
                            
                            if let Some(update) = response {
                                let json = serde_json::to_string(&update).unwrap();
                                let _ = ws_tx.send(warp::ws::Message::text(json)).await;
                            }
                            continue;
                        }
                    }
                }
                
                // MOVIMENTO
                if user_id.is_some() && entity_id.is_some() {
                    let eid = entity_id.unwrap();
                    
                    if DEBUG_MODE {
                        if let Ok(simple_msg) = serde_json::from_str::<ClientMessage>(text) {
                            let response = {
                                let mut game = game_state.lock().unwrap();
                                match simple_msg {
                                    ClientMessage::Move { dx, dy } => {
                                        let old_pos = game.world.get_entity(eid)
                                            .map(|e| e.pos)
                                            .unwrap_or(Position::new(0, 0));
                                        
                                        game.world.move_entity(eid, dx, dy);
                                        
                                        let new_pos = game.world.get_entity(eid)
                                            .map(|e| e.pos)
                                            .unwrap_or(Position::new(0, 0));
                                        
                                        // 🔔 BROADCAST MOVIMENTO
                                        if old_pos.x != new_pos.x || old_pos.y != new_pos.y {
                                            event_hub.lock().unwrap().broadcast(GameEvent::PlayerMoved {
                                                name: player_name.clone(),
                                                from: old_pos,
                                                to: new_pos,
                                            });
                                        }
                                        
                                        game.get_viewport(eid, 15).map(|viewport| {
                                            GameUpdate {
                                                tick: game.world.get_current_tick(),
                                                viewport,
                                            }
                                        })
                                    }
                                    _ => None,
                                }
                            };

                            if let Some(update) = response {
                                let json = serde_json::to_string(&update).unwrap();
                                let _ = ws_tx.send(warp::ws::Message::text(json)).await;
                            }
                        }
                    } 
                    // Modo PRODUÇÃO: mensagem autenticada
                    else {
                        if let Ok(auth_msg) = serde_json::from_str::<AuthenticatedMessage>(text) {
                            let response = {
                                let mut game = game_state.lock().unwrap();
                                match auth_msg.message {
                                    ClientMessage::Move { dx, dy } => {
                                        game.world.move_entity(eid, dx, dy);
                                        game.get_viewport(eid, 15).map(|viewport| {
                                            GameUpdate {
                                                tick: game.world.get_current_tick(),
                                                viewport,
                                            }
                                        })
                                    }
                                    _ => None,
                                }
                            }; // Lock LIBERADO AQUI

                            if let Some(update) = response {
                                let json = serde_json::to_string(&update).unwrap();
                                let _ = ws_tx.send(warp::ws::Message::text(json)).await;
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 🔔 BROADCAST DESCONEXÃO
    if user_id.is_some() {
        event_hub.lock().unwrap().broadcast(GameEvent::PlayerDisconnected {
            name: player_name,
            id: user_id.unwrap().to_string(),
        });
    }
}

/// Stream de eventos para admin-server
async fn stream_events_to_admin(
    ws: warp::ws::WebSocket,
    event_hub: SharedEventHub,
) {
    use futures::{StreamExt, SinkExt};
    
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut event_rx = event_hub.lock().unwrap().subscribe();
    
    // Envia eventos continuamente
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            if ws_tx.send(warp::ws::Message::text(json)).await.is_err() {
                break;
            }
        }
    });
    
    // Mantém conexão aberta
    while let Some(_) = ws_rx.next().await {
        // Admin pode enviar comandos aqui no futuro
    }
}
