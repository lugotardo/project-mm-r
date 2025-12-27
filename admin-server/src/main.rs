use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use warp::Filter;

type EventBroadcaster = Arc<Mutex<broadcast::Sender<String>>>;

#[tokio::main]
async fn main() {
    println!("🖥️  MM Admin Server Starting...");

    let (event_tx, _) = broadcast::channel::<String>(1000);
    let event_tx = Arc::new(Mutex::new(event_tx));
    let event_tx_clone = event_tx.clone();

    tokio::spawn(async move {
        connect_to_game_server(event_tx_clone).await;
    });

    let event_tx_filter = warp::any().map(move || event_tx.clone());
    
    // === CORS Configuration ===
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type"]);
    
    let ws_route = warp::path!("ws" / "events")
        .and(warp::ws())
        .and(event_tx_filter)
        .map(|ws: warp::ws::Ws, tx: EventBroadcaster| {
            ws.on_upgrade(move |socket| handle_admin_websocket(socket, tx))
        });

    let static_files = warp::fs::dir("./admin-panel");

    let routes = ws_route
        .or(static_files)
        .with(cors);  // ← Adiciona CORS

    println!("🖥️  Admin Panel: http://127.0.0.1:3030");
    println!("🔗 Aguardando conexão ao Game Server...");
    println!("✅ CORS habilitado");
    
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn connect_to_game_server(event_tx: EventBroadcaster) {
    loop {
        match connect_async("ws://127.0.0.1:8080/api/events/stream").await {
            Ok((ws_stream, _)) => {
                println!("✅ Admin-Server conectado ao Game-Server");
                let (_write, mut read) = ws_stream.split();

                while let Some(msg) = read.next().await {
                    if let Ok(Message::Text(text)) = msg {
                        println!("📡 Evento do Game-Server: {}", text);

                        // Broadcast para todos os admin-panels conectados
                        let tx = event_tx.lock().unwrap();
                        let _ = tx.send(text);
                    }
                }

                println!("⚠️  Desconectado do Game-Server");
            }
            Err(e) => {
                println!("❌ Erro ao conectar ao Game-Server: {:?}", e);
                println!("🔄 Tentando novamente em 5 segundos...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle_admin_websocket(
    ws: warp::ws::WebSocket,
    event_tx: EventBroadcaster,
) {
    use futures::{SinkExt, StreamExt};

    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut event_rx = event_tx.lock().unwrap().subscribe();

    println!("👤 Admin Panel conectado");

    // Task para enviar eventos
    let send_task = tokio::spawn(async move {
        while let Ok(event_text) = event_rx.recv().await {
            if ws_tx.send(warp::ws::Message::text(event_text)).await.is_err() {
                break;
            }
        }
    });

    // Task para receber comandos do admin (futuro)
    let recv_task = tokio::spawn(async move {
        while let Some(_) = ws_rx.next().await {
            // Admin pode enviar comandos aqui no futuro
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    println!("👤 Admin Panel desconectado");
}
