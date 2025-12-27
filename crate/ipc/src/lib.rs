use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};
use common::Position;

/// Eventos que acontecem no jogo
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GameEvent {
    PlayerConnected { 
        name: String, 
        id: String,
    },
    PlayerDisconnected { 
        name: String, 
        id: String,
    },
    PlayerMoved { 
        name: String, 
        from: Position, 
        to: Position,
    },
    PlayerSpawned {
        name: String,
        pos: Position,
    },
    WorldTick { 
        tick: u64,
        active_players: usize,
        total_entities: usize,
    },
}

/// Hub central de eventos
pub struct EventHub {
    tx: broadcast::Sender<GameEvent>,
}

impl EventHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self { tx }
    }

    pub fn broadcast(&self, event: GameEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GameEvent> {
        self.tx.subscribe()
    }
}

pub type SharedEventHub = Arc<Mutex<EventHub>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_hub() {
        let hub = EventHub::new();
        let mut rx = hub.subscribe();
        
        hub.broadcast(GameEvent::WorldTick { 
            tick: 1, 
            active_players: 0,
            total_entities: 0,
        });
        
        // Não bloqueia em teste síncrono
        assert!(rx.try_recv().is_ok());
    }
}
