use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub channel_ids: Vec<String>,
    pub pushover_token: String,
    pub pushover_user : String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config.toml"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}
