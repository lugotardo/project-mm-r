use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use common::*;
use world::World;
use std::collections::HashMap;

pub type SharedGameState = Arc<Mutex<GameState>>;

pub struct GameState {
    pub world: World,
    pub players: HashMap<u32, PlayerSession>,
    pub update_tx: broadcast::Sender<GameUpdate>,
    pub tick_update_tx: broadcast::Sender<GameUpdate>,  // Novo canal para ticks
}

pub struct PlayerSession {
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
    pub terrain_type: String,
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
        let (tx, _) = broadcast::channel(100);
        let (tick_tx, _) = broadcast::channel(100);
        Self {
            world: World::new(50, 50),
            players: HashMap::new(),
            update_tx: tx,
            tick_update_tx: tick_tx,
        }
    }

    pub fn spawn_player(&mut self, name: String) -> Option<u32> {
        if let Some(entity_id) = self.world.spawn_entity(
            name.clone(),
            Position::new(10, 10),
            EntityType::Player,
        ) {
            self.players.insert(entity_id, PlayerSession {
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

        // Gera tiles visíveis
        for y in (center.y - view_range)..=(center.y + view_range) {
            for x in (center.x - view_range)..=(center.x + view_range) {
                let pos = Position::new(x, y);
                if let Some(tile) = self.world.get_tile(pos) {
                    let (glyph, fg, bg) = tile_to_glyph(&tile);
                    tiles.push(TileData {
                        x,
                        y,
                        glyph,
                        fg_color: fg,
                        bg_color: bg,
                        terrain_type: format!("{:?}", tile.terrain),
                    });
                } else {
                    // Tile fora do mundo
                    tiles.push(TileData {
                        x,
                        y,
                        glyph: ' ',
                        fg_color: "#000".to_string(),
                        bg_color: "#000".to_string(),
                        terrain_type: "Void".to_string(),
                    });
                }
            }
        }

        // Adiciona entidades visíveis
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
    
    /// Envia update para todos os jogadores conectados
    pub fn broadcast_tick_updates(&self) {
        for (player_id, _session) in &self.players {
            if let Some(viewport) = self.get_viewport(*player_id, 15) {
                let update = GameUpdate {
                    tick: self.world.get_current_tick(),
                    viewport,
                };
                let _ = self.tick_update_tx.send(update);
            }
        }
    }
}

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

pub async fn start_game_server(game_state: SharedGameState) {
    let game_state_filter = warp::any().map(move || game_state.clone());

    let static_files = warp::fs::dir("./web-client");

    let ws_game = warp::path!("ws" / "game")
        .and(warp::ws())
        .and(game_state_filter.clone())
        .map(|ws: warp::ws::Ws, state: SharedGameState| {
            ws.on_upgrade(move |websocket| handle_game_websocket(websocket, state))
        });

    let routes = ws_game.or(static_files);

    println!("🎮 Cliente Web: http://127.0.0.1:8081");
    warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
}

async fn handle_game_websocket(ws: warp::ws::WebSocket, game_state: SharedGameState) {
    use futures::{StreamExt, SinkExt};
    
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut player_id: Option<u32> = None;
    
    // Inscreve no canal de updates de tick
    let mut tick_rx = game_state.lock().unwrap().tick_update_tx.subscribe();

    // Task para enviar updates automáticos de tick
    let ws_tx_clone = ws_tx.clone();
    let send_task = tokio::spawn(async move {
        let mut ws_sender = ws_tx_clone;
        while let Ok(update) = tick_rx.recv().await {
            let json = serde_json::to_string(&update).unwrap();
            if ws_sender.send(warp::ws::Message::text(json)).await.is_err() {
                break;
            }
        }
    });

    // Task para receber comandos do cliente
    let recv_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            if let Ok(msg) = result {
                if let Ok(text) = msg.to_str() {
                    if let Ok(cmd) = serde_json::from_str::<ClientMessage>(text) {
                        let response = {
                            let mut state = game_state.lock().unwrap();
                            
                            match cmd {
                                ClientMessage::Login { player_name } => {
                                    if let Some(id) = state.spawn_player(player_name) {
                                        player_id = Some(id);
                                        
                                        if let Some(viewport) = state.get_viewport(id, 15) {
                                            Some(GameUpdate {
                                                tick: state.world.get_current_tick(),
                                                viewport,
                                            })
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                                ClientMessage::Move { dx, dy } => {
                                    if let Some(id) = player_id {
                                        state.world.move_entity(id, dx, dy);
                                        
                                        if let Some(viewport) = state.get_viewport(id, 15) {
                                            Some(GameUpdate {
                                                tick: state.world.get_current_tick(),
                                                viewport,
                                            })
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                            }
                        };
                        
                        if let Some(update) = response {
                            let json = serde_json::to_string(&update).unwrap();
                            if ws_tx.send(warp::ws::Message::text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_creation() {
        let state = GameState::new();
        assert_eq!(state.players.len(), 0);
    }

    #[test]
    fn test_spawn_player() {
        let mut state = GameState::new();
        let id = state.spawn_player("TestPlayer".to_string());
        assert!(id.is_some());
        assert_eq!(state.players.len(), 1);
    }
}
