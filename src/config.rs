use crate::constants::*;
use serde::Deserialize;

pub mod const_conf {
    use super::*;

    #[derive(Deserialize, Clone)]
    pub struct ConstConfig {
        ollama: OllamaSettings,
        telegram: TelegramSettings,
        database: DatabaseSettings,
    }

    #[derive(Deserialize, Clone)]
    pub struct OllamaSettings {
        url: Option<String>,
    }

    #[derive(Deserialize, Clone)]
    pub struct TelegramSettings {
        callsign: String,
        token: String,
    }
    
    #[derive(Deserialize, Clone)]
    pub struct DatabaseSettings {
        db: String,
    }

    impl ConstConfig {
        pub fn get() -> Result<Self, toml::de::Error> {
            toml::from_str(COST_CFG)
        }

        pub fn get_ollama_settings(&self) -> &OllamaSettings {
            &self.ollama
        }

        pub fn get_telegram_settings(&self) -> &TelegramSettings {
            &self.telegram
        }

        pub fn get_database_settings(&self) -> &DatabaseSettings {
            &self.database
        }
    }

    impl TelegramSettings {
        pub fn get_token(&self) -> &str {
            &self.token
        }

        pub fn get_callsign(&self) -> &str {
            Box::new(self.callsign.to_lowercase()).leak()
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
}

pub mod dyn_conf {
    use super::*;

    #[derive(Deserialize, Clone)]
    pub struct DynConfig {
        ollama: OllamaSettings,
    }

    #[derive(Deserialize, Clone)]
    pub struct OllamaSettings {
        model: String,
        thinking: bool,
        sys_prompt: String,
    }

    #[derive(Debug)]
    pub enum ConfigError {
        TomlError(toml::de::Error),
        IOError(std::io::Error),
    }

    impl std::fmt::Display for ConfigError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::TomlError(e) => write!(f, "Failed to parse dynamic config: {e}"),
                Self::IOError(e) => write!(f, "Failed to read dynamic config: {e}"),
            }
        }
    }

    impl std::error::Error for ConfigError {}

    impl DynConfig {
        pub fn get() -> Result<Self, ConfigError> {
            Ok(
                toml::from_str(
                    &std::fs::read_to_string(DYNM_CFG_PATH).map_err(ConfigError::IOError)?,
                )
                .map_err(ConfigError::TomlError)?
            )
        }

        pub fn get_ollama_settings(&self) -> &OllamaSettings {
            &self.ollama
        }
    }

    impl OllamaSettings {
        pub fn get_model(&self) -> String {
            self.model.clone()
        }

        pub fn get_system_prompt(&self) -> String {
            self.sys_prompt.clone()
        }

        pub fn get_thinking(&self) -> bool {
            self.thinking.clone()
        }
    }
}