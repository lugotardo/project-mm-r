use common::*;
use std::io::{self, Write};
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect("127.0.0.1:4000")?;
    println!("🎮 Cliente conectado ao servidor");

    // Login
    print!("Digite seu nome: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim().to_string();

    let login_msg = ClientMessage::Login { player_name: name };
    let data = serde_json::to_vec(&login_msg).unwrap();
    socket.send(&data)?;

    // Recebe confirmação
    let mut buf = [0u8; 4096];
    let amt = socket.recv(&mut buf)?;
    if let Ok(ServerMessage::ActionResult { success, message }) = 
        serde_json::from_slice(&buf[..amt]) {
        if success {
            println!("✅ {}", message);
        }
    }

    println!("\n📋 Comandos: W/A/S/D (mover) | Q (sair)\n");

    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_uppercase();

        let msg = match input.as_str() {
            "W" => ClientMessage::Move { dx: 0, dy: -1 },
            "S" => ClientMessage::Move { dx: 0, dy: 1 },
            "A" => ClientMessage::Move { dx: -1, dy: 0 },
            "D" => ClientMessage::Move { dx: 1, dy: 0 },
            "Q" => {
                println!("👋 Saindo...");
                break;
            }
            _ => {
                println!("❌ Comando inválido");
                continue;
            }
        };

        let data = serde_json::to_vec(&msg).unwrap();
        socket.send(&data)?;

        // Recebe update do mundo
        let amt = socket.recv(&mut buf)?;
        if let Ok(ServerMessage::WorldUpdate { entities, .. }) = 
            serde_json::from_slice(&buf[..amt]) {
            println!("\n🌍 Estado do mundo:");
            for entity in entities {
                println!("  - {} (ID: {}) na posição ({}, {})", 
                    entity.name, entity.id, entity.pos.x, entity.pos.y);
            }
            println!();
        }
    }

    Ok(())
}
