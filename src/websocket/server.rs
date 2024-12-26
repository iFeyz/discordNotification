use features_utils::{Sink, StreamExt};
use tokio::net::{TcpListener , TcpStream};
use tokio_tungstenite::accept_async;
use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::Mutex;
use features::channel::mpsc::{channel , Sender};

pub type WsMessageSender = Sender<String>;
type ConnectedClients = Arc<Mutex<HashSet<WsMessageSender>>>;

pub struct WsServer {
    clients: ConnectedClients,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashSet::new()))
        }
    }

    pub async fn start(&self , addr : &str) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("WebSocket server started on {}", addr);

        while let Ok((stream , _)) = listener.accept().await {
            let clients = self.clients.clone();
            tokio::spawn(handle_connection(stream , clients));
        }

        Ok(())
    }

    pub async fn broadcast(&self, message: &str) -> anyhow::Result<()> {
        let mut clients = self.clients.lock().await;
        clients.retain_mut(|sender| {
            sender.try_send(message.to_string()).is_ok()
        });
        Ok(())
    }
}

async fn handle_connection(stream : TcpStream , clients : ConnectedClients) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept connection");
    let (mut ws_sender , ws_receiver) = ws_stream.split();
    let (client_sender , client_receiver) = channel(32);

    clients.lock().await.insert(client_sender);

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(_)) => {
                        // Handleling message from client not need here
                    }

                    _=> break,
                }
            }
            msg = client_receiver.next() => {
                match msg {
                    Some(msg) => {
                        if ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }

    clients.lock().await.retain(|sender| sender != &client_sender);
}