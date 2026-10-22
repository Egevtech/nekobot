use serde::{Deserialize};
use crate::constants;
use rhai::{Engine, Dynamic};

#[derive(Deserialize)]
pub struct DynConfig {
    pub ollama: OllamaSettings,
}

#[derive(Deserialize)]
pub struct OllamaSettings {
    model: String,
    sys_prompt: String,
    thinking: bool,
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

impl DynConfig {
    pub fn get() -> Self {
        let engine: Engine = Engine::new();

        let conf_dyn = engine.eval_file::<Dynamic>(std::path::Path::new(constants::DYNM_CFG_PATH).into()).expect("Failed to read dynamic configuration");

        rhai::serde::from_dynamic(&conf_dyn).expect("Failed to parse dynamic config")
    }
}