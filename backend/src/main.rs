mod admin;

use common::*;
use world::World;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use admin::{AdminState, AdminEvent, SharedAdminState};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Inicializa estado admin
    let admin_state: SharedAdminState = Arc::new(Mutex::new(AdminState::new()));
    let admin_state_clone = admin_state.clone();

    // Inicia servidor admin em thread separada
    tokio::spawn(async move {
        admin::start_admin_server(admin_state_clone).await;
    });

    // Servidor de jogo (UDP)
    let socket = UdpSocket::bind("127.0.0.1:4000")?;
    socket.set_nonblocking(false)?;
    
    println!("🌍 Servidor rodando em 127.0.0.1:4000");
    println!("📐 Mundo: 20x20 tiles com camadas");

    let mut world = World::new(20, 20);
    let mut clients: HashMap<std::net::SocketAddr, u32> = HashMap::new();
    let mut buf = [0u8; 4096];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        
        match serde_json::from_slice::<ClientMessage>(&buf[..amt]) {
            Ok(msg) => {
                match msg {
                    ClientMessage::Login { player_name } => {
                        if let Some(entity_id) = world.spawn_entity(
                            player_name.clone(),
                            Position::new(3, 3),
                            EntityType::Player
                        ) {
                            clients.insert(src, entity_id);
                            println!("✅ Jogador '{}' conectado (ID: {})", player_name, entity_id);
                            
                            // Log para admin
                            admin_state.lock().unwrap().log_event(
                                AdminEvent::PlayerConnected {
                                    name: player_name.clone(),
                                    id: entity_id,
                                }
                            );
                            
                            let response = ServerMessage::ActionResult {
                                success: true,
                                message: format!("Bem-vindo, {}!", player_name),
                            };
                            let data = serde_json::to_vec(&response).unwrap();
                            socket.send_to(&data, src)?;
                        } else {
                            let response = ServerMessage::ActionResult {
                                success: false,
                                message: "Falha ao spawnar jogador".to_string(),
                            };
                            let data = serde_json::to_vec(&response).unwrap();
                            socket.send_to(&data, src)?;
                        }
                    }
                    ClientMessage::Move { dx, dy } => {
                        if let Some(&entity_id) = clients.get(&src) {
                            if let Some(entity) = world.get_entity(entity_id) {
                                let old_pos = entity.pos;
                                let success = world.move_entity(entity_id, dx, dy);
                                
                                if success {
                                    if let Some(entity) = world.get_entity(entity_id) {
                                        println!("🚶 {} moveu para ({}, {})", 
                                            entity.name, entity.pos.x, entity.pos.y);
                                        
                                        // Log para admin
                                        admin_state.lock().unwrap().log_event(
                                            AdminEvent::PlayerMoved {
                                                name: entity.name.clone(),
                                                from: old_pos,
                                                to: entity.pos,
                                            }
                                        );
                                    }
                                }
                            }
                            
                            // Envia snapshot do mundo visível
                            if let Some(entity) = world.get_entity(entity_id) {
                                let snapshot = world.get_visible_snapshot(entity.pos, 7);
                                let update = ServerMessage::WorldUpdate {
                                    tiles: snapshot.tiles,
                                    entities: snapshot.entities,
                                };
                                let data = serde_json::to_vec(&update).unwrap();
                                socket.send_to(&data, src)?;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Erro ao deserializar mensagem: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_integration() {
        let mut world = World::new(10, 10);
        let id = world.spawn_entity(
            "TestPlayer".to_string(),
            Position::new(3, 3),
            EntityType::Player
        );
        assert!(id.is_some());
    }
}
