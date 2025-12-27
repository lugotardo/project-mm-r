use world::World;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("🌍 MM World Simulator Starting...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Este processo simula o mundo continuamente:");
    println!("  • NPCs com IA");
    println!("  • Eventos aleatórios");
    println!("  • Crescimento de facções");
    println!("  • História emergente");
    println!("  • Mundo persiste mesmo sem jogadores");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let mut world = World::new(100, 100);
    let mut tick_counter = 0u64;

    println!("✅ Mundo criado: 100x100 tiles");
    println!("🎯 Iniciando loop de simulação (1 tick/segundo)...");
    println!();

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        world.tick();
        tick_counter += 1;

        // Log a cada 10 ticks
        if tick_counter % 10 == 0 {
            println!("🔄 Tick #{:4} | Entidades: {:3}", 
                tick_counter,
                world.entity_count()
            );
        }

        // Eventos históricos a cada 100 ticks
        if tick_counter % 100 == 0 {
            let events = world.get_historical_events(5);
            if !events.is_empty() {
                println!("📜 Últimos eventos:");
                for event in events.iter().take(3) {
                    println!("   └─ {}", event.description);
                }
            }
        }

        // Status detalhado a cada 1000 ticks (~16 minutos)
        if tick_counter % 1000 == 0 {
            println!();
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("📊 STATUS DO MUNDO (Tick {})", tick_counter);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            let (w, h) = world.dimensions();
            println!("   Dimensões: {}x{}", w, h);
            println!("   Entidades ativas: {}", world.entity_count());
            println!("   Eventos registrados: {}", world.get_historical_events(99999).len());
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
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
