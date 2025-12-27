use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use common::*;
use std::time::Instant;

/// Estatísticas do servidor
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerStats {
    pub uptime_seconds: u64,
    pub total_players: usize,
    pub active_players: usize,
    pub world_size: (i32, i32),
    pub total_entities: usize,
    pub total_tiles: usize,
    pub ticks_processed: u64,
}

/// Eventos do servidor para admin
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AdminEvent {
    PlayerConnected { name: String, id: u32 },
    PlayerDisconnected { name: String, id: u32 },
    PlayerMoved { name: String, from: Position, to: Position },
    EntitySpawned { entity_type: String, pos: Position },
    CombatOccurred { attacker: String, defender: String, pos: Position },
    WorldTick { tick: u64 },
    ServerStats(ServerStats),
}

/// Estado compartilhado do admin
pub struct AdminState {
    pub events: Vec<AdminEvent>,
    pub max_events: usize,
    pub event_tx: broadcast::Sender<AdminEvent>,
    pub start_time: Instant,
}

impl AdminState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            events: Vec::new(),
            max_events: 1000,
            event_tx: tx,
            start_time: Instant::now(),
        }
    }

    pub fn log_event(&mut self, event: AdminEvent) {
        self.events.push(event.clone());
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
        let _ = self.event_tx.send(event);
    }

    pub fn get_recent_events(&self, limit: usize) -> Vec<AdminEvent> {
        let start = if self.events.len() > limit {
            self.events.len() - limit
        } else {
            0
        };
        self.events[start..].to_vec()
    }
    
    pub fn get_uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

pub type SharedAdminState = Arc<Mutex<AdminState>>;

// Estrutura para compartilhar stats do jogo
#[derive(Clone)]
pub struct GameStats {
    pub world_size: (i32, i32),
    pub total_entities: usize,
    pub active_players: usize,
    pub current_tick: u64,
}

/// Inicia servidor admin HTTP + WebSocket
pub async fn start_admin_server(
    admin_state: SharedAdminState,
    stats_rx: tokio::sync::watch::Receiver<GameStats>,
) {
    let admin_state_filter = warp::any().map(move || admin_state.clone());
    let stats_filter = warp::any().map(move || stats_rx.clone());

    // Rota: GET /api/stats
    let stats_route = warp::path!("api" / "stats")
        .and(admin_state_filter.clone())
        .and(stats_filter.clone())
        .map(|admin: SharedAdminState, stats: tokio::sync::watch::Receiver<GameStats>| {
            let admin = admin.lock().unwrap();
            let game_stats = stats.borrow().clone();
            
            let response = ServerStats {
                uptime_seconds: admin.get_uptime_seconds(),
                total_players: game_stats.active_players,
                active_players: game_stats.active_players,
                world_size: game_stats.world_size,
                total_entities: game_stats.total_entities,
                total_tiles: (game_stats.world_size.0 * game_stats.world_size.1) as usize,
                ticks_processed: game_stats.current_tick,
            };
            
            warp::reply::json(&response)
        });

    // Rota: GET /api/events?limit=50
    let events_route = warp::path!("api" / "events")
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(admin_state.clone())
        .map(|params: std::collections::HashMap<String, String>, state: SharedAdminState| {
            let limit = params.get("limit")
                .and_then(|s| s.parse().ok())
                .unwrap_or(50);
            let state = state.lock().unwrap();
            let events = state.get_recent_events(limit);
            warp::reply::json(&events)
        });

    // Rota: WebSocket /ws/events
    let ws_route = warp::path!("ws" / "events")
        .and(warp::ws())
        .and(admin_state.clone())
        .map(|ws: warp::ws::Ws, state: SharedAdminState| {
            ws.on_upgrade(move |websocket| handle_websocket(websocket, state))
        });

    // Servir arquivos estáticos (HTML/CSS/JS)
    let static_files = warp::fs::dir("./admin-panel");

    let routes = stats_route
        .or(events_route)
        .or(ws_route)
        .or(static_files);

    println!("🖥️  Painel Admin: http://127.0.0.1:3031");
    warp::serve(routes).run(([127, 0, 0, 1], 3031)).await;
}

/// Handler do WebSocket
async fn handle_websocket(ws: warp::ws::WebSocket, admin_state: SharedAdminState) {
    use futures::{StreamExt, SinkExt};
    
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut event_rx = admin_state.lock().unwrap().event_tx.subscribe();

    // Task para enviar eventos ao cliente
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            if ws_tx.send(warp::ws::Message::text(json)).await.is_err() {
                break;
            }
        }
    });

    // Task para receber mensagens do cliente (se necessário)
    let recv_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            if result.is_err() {
                break;
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
    fn test_admin_state_creation() {
        let state = AdminState::new();
        assert_eq!(state.events.len(), 0);
    }

    #[test]
    fn test_log_event() {
        let mut state = AdminState::new();
        state.log_event(AdminEvent::WorldTick { tick: 1 });
        assert_eq!(state.events.len(), 1);
    }

    #[test]
    fn test_event_limit() {
        let mut state = AdminState::new();
        state.max_events = 3;
        
        for i in 0..5 {
            state.log_event(AdminEvent::WorldTick { tick: i });
        }
        
        assert_eq!(state.events.len(), 3);
    }
}
