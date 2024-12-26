use anyhow::Result;
use serde::Deserialize;
use crate::discord::client::DiscordClient;
use crate::notification::pushover::PushoverNotifier;
use config::Config as ConfigLoader;

mod discord;
mod notification;

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
    let mut discord = DiscordClient::new(config.discord_token).await?;

    println!("Bot démarré ! En écoute sur les channels configurés...");

    discord
        .run(
            config.channel_ids,
            |text| notifier.send(text)
        )
        .await?;

    Ok(())
}