use crate::constants::*;
use serde::Deserialize;


#[derive(Deserialize, Clone)]
pub struct ConstConfig {
    pub ollama: OllamaSettings,
    pub telegram: TelegramSettings,
    pub database: DatabaseSettings,
}

#[derive(Deserialize, Clone)]
pub struct OllamaSettings {
    url: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct TelegramSettings {
    callsign: String,
    token: String,
    message_length: usize,
}
    
#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    db: String,
}

impl ConstConfig {
    pub fn get() -> Result<Self, toml::de::Error> {
        toml::from_str(COST_CFG)
    }
}

impl TelegramSettings {
    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub fn get_callsign(&self) -> &str {
        Box::new(self.callsign.to_lowercase()).leak()
    }

    pub fn get_message_length(&self) -> usize{
        self.message_length
    }
}

impl OllamaSettings {
    pub fn get_url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}

impl DatabaseSettings {
    pub fn get_db_path(&self) -> String {
        self.db.clone()
    }
}
