use twilight_gateway::{Event, Intents, Shard};
use twilight_gateway::ShardId;
use anyhow::Result;
use std::sync::Arc;
use crate::websocket::server::WebSocketServer;

pub struct DiscordClient {
    shard: Shard,
}

impl DiscordClient {
    pub async fn new(token: String) -> Result<Self> {
        let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
        let shard = Shard::new(ShardId::new(0, 1), token, intents);
        Ok(Self { shard })
    }

    pub async fn run(
        &mut self,
        channel_ids: Vec<String>,
        notification_sender: impl Fn(&str) -> Result<()>,
        ws_server: Arc<WebSocketServer>,
    ) -> Result<()> {
        println!("Bot is running! Listening for messages...");

        loop {
            let event = match self.shard.next_event().await {
                Ok(event) => event,
                Err(error) => {
                    println!("Error receiving event: {:?}", error);
                    continue;
                }
            };

            if let Event::MessageCreate(msg) = event {
                if channel_ids.contains(&msg.channel_id.to_string()) {
                    println!("Message reçu dans #{}: {}: {}", 
                        msg.channel_id, 
                        msg.author.name, 
                        msg.content
                    );
                    
                    let notification_text = format!(
                        "#{}: {}: {}",
                        msg.channel_id, msg.author.name, msg.content
                    );
                    
                    if let Err(e) = notification_sender(&notification_text) {
                        println!("Erreur d'envoi de notification: {:?}", e);
                    }

                    if let Err(e) = ws_server.broadcast(&notification_text).await {
                        println!("Erreur lors de la diffusion du message: {:?}", e);
                    }
                }
            }
        }
    }
}