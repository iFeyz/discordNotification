use pushover::requests::message::SendMessage;
use pushover::API;
use anyhow::Result;

pub struct PushoverNotifier {
    api: API,
    user: String,
    token: String,
}

impl PushoverNotifier {
    pub fn new(token: String, user: String) -> Self {
        Self {
            api: API::new(),
            user,
            token,
        }
    }

    pub fn send(&self, text: &str) -> Result<()> {
        // Créons le message avec les paramètres requis
        let msg = SendMessage::new(
            &self.token,
            &self.user,
            text,
        );
        
        // Envoyons le message
        self.api.send(&msg)
            .map_err(|e| anyhow::anyhow!("Failed to send Pushover notification: {}", e))?;
        
        Ok(())
    }
}