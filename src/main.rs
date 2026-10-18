pub mod config;
pub mod constants;

use std::sync::Arc;
use tokio::sync::Mutex;

use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use teloxide::{
    dispatching::{Dispatcher, dialogue::GetChatId},
    prelude::*,
    types::ReplyParameters,
};

use config::*;

#[tokio::main]
async fn main() {
    let const_config = const_conf::ConstConfig::get().expect("Failed to read configuration");
    let bot = Bot::new(const_config.get_telegram_settings().get_token());
    let busy = Arc::new(Mutex::new(()));

    let ollama;
    if const_config.get_ollama_settings().has_url() {
        ollama =
            Ollama::try_new(const_config.get_ollama_settings().get_url().unwrap()).expect("Failed to create ollama instance");
    } else {
        ollama = 
            Ollama::default();
    }

    let handler = Update::filter_message().endpoint(move |bot: Bot, msg: Message| {
        type Hyeta = Result<(), teloxide::RequestError>;

        let ollama = ollama.clone();
        let const_config = const_config.clone();
        let busy = busy.clone();

        async move {
            let dyn_config = match dyn_conf::DynConfig::get() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Preparing dynamic configuration failed: {e}");
                    return Hyeta::Ok(());
                }
            };

            if msg.text().is_none() || msg.chat_id().is_none() || msg.from.is_none() {
                return Hyeta::Ok(());
            }

            let chat_id = msg.chat_id().unwrap();
            let text = msg.text().unwrap().to_string();
            let msg_parts = text
                .split(" ")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            if msg_parts.len() < 2 || msg_parts[0].to_lowercase() != const_config.get_telegram_settings().get_callsign() {
                return Hyeta::Ok(());
            }

            let Ok(_guard) = busy.try_lock() else {
                bot.send_message(chat_id, "Я уже отвечаю на другой запрос...")
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
                return Hyeta::Ok(());
            };

            bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing)
                .await?;

            let prompt = msg_parts[1..].join(" ");
            println!("Request from {}: {}", msg.from.unwrap().first_name, prompt);

            let req = GenerationRequest::new(dyn_config.get_ollama_settings().get_model(), prompt)
                .think(false)
                .system(dyn_config.get_ollama_settings().get_system_prompt());

            match ollama.generate(req).await {
                Ok(res) => {
                    bot.send_message(chat_id, res.response)
                        .reply_parameters(ReplyParameters::new(msg.id))
                        .await?
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Ошибка генерации: {}", e))
                        .reply_parameters(ReplyParameters::new(msg.id))
                        .await?
                }
            };

            Hyeta::Ok(())
        }
    });

    println!("Ready");

    Dispatcher::builder(bot, handler)
        .distribution_function(|_| None::<()>)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
