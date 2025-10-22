use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use warp::ws::Message;
use warp::filters::ws::{WebSocket, Ws};
use warp::Filter;
use futures_util::{SinkExt, StreamExt};

use crate::interface::cli::logging::Logger;

/// WebSocket server for live reload functionality
#[derive(Clone)]
pub struct WebSocketServer {
    clients: Arc<RwLock<HashMap<String, broadcast::Sender<()>>>>,
    port: u16,
}

impl WebSocketServer {
    pub fn new(port: u16) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            port,
        }
    }

    /// Create WebSocket filter for warp
    pub fn websocket_filter(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let clients = self.clients.clone();
        let port = self.port;

        warp::path("ws")
            .and(warp::ws())
            .map(move |ws: Ws| {
                let clients = clients.clone();
                ws.on_upgrade(move |websocket| {
                    Self::handle_websocket_connection(websocket, clients, port)
                })
            })
    }

    /// Handle individual WebSocket connection
    async fn handle_websocket_connection(
        websocket: WebSocket,
        clients: Arc<RwLock<HashMap<String, broadcast::Sender<()>>>>,
        port: u16,
    ) {
        let client_id = uuid7::uuid7().to_string();
        let (tx, mut rx) = broadcast::channel(10);
        
        // Store client
        {
            let mut clients_guard = clients.write().await;
            clients_guard.insert(client_id.clone(), tx.clone());
        }

        Logger::info(&format!("WebSocket client connected: {}", client_id));

        // Split WebSocket into sender and receiver
        let (ws_sender, mut ws_receiver) = websocket.split();
        let ws_sender = std::sync::Arc::new(tokio::sync::Mutex::new(ws_sender));

        // Handle incoming messages
        let clients_clone = clients.clone();
        let client_id_clone = client_id.clone();
        let ws_sender_clone = ws_sender.clone();
        let handle_incoming = tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(msg) => {
                        if msg.is_text() {
                            let text = msg.to_str().unwrap_or("");
                            Logger::debug(&format!("Received from {}: {}", client_id_clone, text));
                            // Handle ping/pong for connection health
                            if text == "ping" {
                                let mut sender = ws_sender_clone.lock().await;
                                if let Err(e) = sender.send(Message::text("pong")).await {
                                    Logger::error(&format!("Failed to send pong: {}", e));
                                    break;
                                }
                            }
                        } else if msg.is_close() {
                            Logger::info(&format!("WebSocket client {} disconnected", client_id_clone));
                            break;
                        }
                    }
                    Err(e) => {
                        Logger::error(&format!("WebSocket error for {}: {}", client_id_clone, e));
                        break;
                    }
                }
            }
        });

        // Handle broadcast messages (reload signals)
        let ws_sender_broadcast = ws_sender.clone();
        let handle_broadcast = tokio::spawn(async move {
            while let Ok(_) = rx.recv().await {
                let mut sender = ws_sender_broadcast.lock().await;
                if let Err(e) = sender.send(Message::text("reload")).await {
                    Logger::error(&format!("Failed to send reload signal: {}", e));
                    break;
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            _ = handle_incoming => {},
            _ = handle_broadcast => {},
        }

        // Clean up client
        {
            let mut clients_guard = clients_clone.write().await;
            clients_guard.remove(&client_id);
        }

        Logger::info(&format!("WebSocket client {} cleaned up", client_id));
    }

    /// Broadcast reload signal to all connected clients
    pub async fn broadcast_reload(&self) {
        let mut disconnected_clients = Vec::new();
        let client_count = {
            let clients_guard = self.clients.read().await;
            for (client_id, tx) in clients_guard.iter() {
                if let Err(_) = tx.send(()) {
                    // Client disconnected, mark for removal
                    disconnected_clients.push(client_id.clone());
                }
            }
            clients_guard.len()
        };

        // Clean up disconnected clients
        if !disconnected_clients.is_empty() {
            let mut clients_guard = self.clients.write().await;
            for client_id in disconnected_clients {
                clients_guard.remove(&client_id);
            }
        }

        Logger::info(&format!("Reload signal sent to {} clients", client_count));
    }

    /// Get number of connected clients
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }
}

/// Live reload script that connects to WebSocket
pub fn get_live_reload_script(port: u16) -> String {
    format!(
        r#"
<script>
(function() {{
    const wsUrl = 'ws://localhost:{}/ws';
    let ws = null;
    let reconnectAttempts = 0;
    const maxReconnectAttempts = 10;
    const reconnectDelay = 1000;

    function connect() {{
        try {{
            ws = new WebSocket(wsUrl);
            
            ws.onopen = function() {{
                console.log('Live reload connected');
                reconnectAttempts = 0;
            }};
            
            ws.onmessage = function(event) {{
                if (event.data === 'reload') {{
                    console.log('Live reload triggered');
                    window.location.reload();
                }} else if (event.data === 'pong') {{
                    console.log('Live reload ping received');
                }}
            }};
            
            ws.onclose = function() {{
                console.log('Live reload disconnected');
                if (reconnectAttempts < maxReconnectAttempts) {{
                    reconnectAttempts++;
                    console.log(`Attempting to reconnect... (${{reconnectAttempts}}/${{maxReconnectAttempts}})`);
                    setTimeout(connect, reconnectDelay * reconnectAttempts);
                }}
            }};
            
            ws.onerror = function(error) {{
                console.error('Live reload WebSocket error:', error);
            }};
        }} catch (error) {{
            console.error('Failed to create WebSocket connection:', error);
        }}
    }}

    // Start connection
    connect();

    // Send ping every 30 seconds to keep connection alive
    setInterval(function() {{
        if (ws && ws.readyState === WebSocket.OPEN) {{
            ws.send('ping');
        }}
    }}, 30000);
}})();
</script>
"#,
        port
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_websocket_server_creation() {
        let server = WebSocketServer::new(3000);
        assert_eq!(server.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_live_reload_script_generation() {
        let script = get_live_reload_script(3000);
        assert!(script.contains("ws://localhost:3000/ws"));
        assert!(script.contains("reload"));
        assert!(script.contains("ping"));
        assert!(script.contains("pong"));
    }
}
