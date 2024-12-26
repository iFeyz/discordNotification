use anyhow::Result;
use serde::Deserialize;
use crate::discord::client::DiscordClient;
use crate::notification::pushover::PushoverNotifier;
use config::Config as ConfigLoader;
use std::sync::Arc;
use crate::websocket::server::WebSocketServer;

mod discord;
mod notification;
mod websocket;

#[derive(Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub pushover_token: String,
    pub pushover_user: String,
    pub channel_ids: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let settings = ConfigLoader::builder()
            .add_source(config::File::with_name("config"))
            .build()?;
            
        let config = settings.try_deserialize()?;
        Ok(config)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;
    let notifier = PushoverNotifier::new(config.pushover_token, config.pushover_user);

    // Server websocket
    let ws_server = Arc::new(WebSocketServer::new());
    let ws_server_clone = ws_server.clone();
    // spawn websocket server
    tokio::spawn(async move {
        if let Err(e) = ws_server_clone.start("127.0.0.1:8090").await {
            println!("Erreur lors du démarrage du serveur WebSocket: {:?}", e);
        }
    });

    let mut discord = DiscordClient::new(config.discord_token).await?;

    println!("Bot démarré ! En écoute sur les channels configurés...");
    println!("WebSocket server running on ws://127.0.0.1:8090");

    discord
        .run(
            config.channel_ids,
            |text| notifier.send(text),
            ws_server
        )
        .await?;

    Ok(())
}