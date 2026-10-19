mod config;
mod constants;
mod db;
mod args;

use std::sync::Arc;
use tokio::sync::Mutex;

use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use teloxide::{
    dispatching::{Dispatcher, dialogue::GetChatId},
    prelude::*,
    types::ReplyParameters,
};

use config::*;
use args::Cmd;

#[tokio::main]
async fn main() {
    let const_config = const_conf::ConstConfig::get().expect("Failed to read configuration");
    let bot = Bot::new(const_config.get_telegram_settings().get_token());
    let busy = Arc::new(Mutex::new(()));

    let args = Cmd::get_args();

    match args.subcommand {
        args::SCommand::RecreateDatabase{recreate_only}=> {
            if std::fs::exists(const_config.get_database_settings().get_db_path()).unwrap() {
                std::fs::remove_file(const_config.get_database_settings().get_db_path()).expect("Failed to remove old database");
            }

            println!("Database removed");

            if recreate_only {
                db::Database::new(const_config.get_database_settings().get_db_path());
                std::process::exit(0);
            }
        }
    }

    let database = db::Database::new(const_config.get_database_settings().get_db_path());
    let arc_db = Arc::new(Mutex::new(database));

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
        let arc_db = arc_db.clone();

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
            let from = msg.from.clone().unwrap();
            let text = msg.text().unwrap().to_string();
            let msg_parts = text
                .split(" ")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            if msg_parts.len() < 2 || msg_parts[0].to_lowercase() != const_config.get_telegram_settings().get_callsign() {
                return Hyeta::Ok(());
            }

            tokio::spawn({
                let arc_db = arc_db.clone();
                let user_id = from.id.0;
                let username = from.username.clone();

                async move {
                    let db = arc_db.lock().await;

                    if !db.is_user_in_database(user_id).expect("Failed to check if user is already in database") {
                        println!("Pushing user {}({}) to database", user_id, username_to_string(&username));

                        db.add_user(user_id).expect("Failed to add user to the database");
                    } else {
                        println!("User {}({}) already in database", user_id, username_to_string(&username));
                    }
                }
            });

            let Ok(_guard) = busy.try_lock() else {
                println!("Request from {}({}) blocked, because bot is already busy", from.first_name, username_to_string(&from.username));
                bot.send_message(chat_id, "Я уже отвечаю на другой запрос...")
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
                return Hyeta::Ok(());
            };

            let chatact_task = tokio::spawn(  {
                let chat_id = chat_id.clone();
                let bot = bot.clone();
                
                async move {
                    loop {
                        if let Err(e) =bot.clone().send_chat_action(chat_id, teloxide::types::ChatAction::Typing).await {
                            eprintln!("ChatAction thread failure: {e}");
                        } 

                        tokio::time::sleep(tokio::time::Duration::from_millis(4000)).await;
                    }
                }
            });

            let prompt = format!("{} пишет: ", username_to_string(&from.username)) + &msg_parts[1..].join(" ");
            println!("Request from {}({}): {}", from.first_name, username_to_string(&from.username), prompt);

            let req = GenerationRequest::new(dyn_config.get_ollama_settings().get_model(), prompt)
                .think(dyn_config.get_ollama_settings().get_thinking())
                .system(dyn_config.get_ollama_settings().get_system_prompt());

            match ollama.generate(req).await {
                Ok(res) => {
                    let mut resp = res.response;

                    if resp.len() > 4096 {
                        resp = cut_string(resp, 4096);
                    }

                    bot.send_message(chat_id, resp)
                        .reply_parameters(ReplyParameters::new(msg.id))
                        .await?
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Ошибка генерации: {}", e))
                        .reply_parameters(ReplyParameters::new(msg.id))
                        .await?
                }
            };

            chatact_task.abort();

            println!("Request done");

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

fn username_to_string(username: &Option<String>) -> String {
    match username {
        Some(s) => format!("@{s}"),
        None => "NO_USERNAME".to_string(),
    }
}
/**
 * `max` - maximum length of `msg`, include '...'
 */
fn cut_string(str: String, max: usize) -> String {
    str.chars().take(max - 3).collect::<String>() + "..."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_message_test() {
        let str = "123456789".to_string();
        let sstr = cut_string(str.clone(), 5); // 12...

        assert_eq!(sstr, "12...");
    }

    #[test]
    fn username_print_test() {
        assert_eq!(&username_to_string(&Some("kaka".to_string())), "@kaka");
        assert_eq!(&username_to_string(&None), "NO_USERNAME");
    }
}