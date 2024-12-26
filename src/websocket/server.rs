use futures_util::{StreamExt, SinkExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::Mutex;
use futures::channel::mpsc::{channel, Sender};
use std::hash::{Hash, Hasher};

pub type WsMessageSender = Sender<String>;

#[derive(Debug)]
struct SenderWrapper(WsMessageSender);

impl PartialEq for SenderWrapper {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.0, &other.0)
    }
}

impl Eq for SenderWrapper {}

impl Hash for SenderWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(&self.0, state);
    }
}

type ConnectedClients = Arc<Mutex<HashSet<SenderWrapper>>>;

pub struct WebSocketServer {
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
        let message = message.to_string();
        clients.retain(|wrapper| {
            let mut sender = wrapper.0.clone();
            sender.try_send(message.clone()).is_ok()
        });
        Ok(())
    }
}

async fn handle_connection(stream : TcpStream , clients : ConnectedClients) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept connection");
    let (mut ws_sender , mut ws_receiver) = ws_stream.split();
    let (client_sender , mut client_receiver) = channel(32);

    clients.lock().await.insert(SenderWrapper(client_sender.clone()));

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

    clients.lock().await.retain(|wrapper| !std::ptr::eq(&wrapper.0, &client_sender));
}