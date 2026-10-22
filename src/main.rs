mod config;
mod constants;
mod db;
mod args;
mod splitby;

use std::sync::Arc;
use tokio::sync::Mutex;

use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use teloxide::{
    dispatching::{Dispatcher, dialogue::GetChatId},
    prelude::*,
    types::ReplyParameters,
};

use config::*;
use splitby::*;
use args::Cmd;

#[tokio::main]
async fn main() {
    let const_config = const_config::ConstConfig::get().expect("Failed to read configuration");
    let bot = Bot::new(const_config.telegram.get_token());
    let busy = Arc::new(Mutex::new(()));

    let args = Cmd::get_args();

    if args.subcommand.is_some() {
        match args.subcommand.unwrap() {
            args::SCommand::RecreateDatabase{recreate_only} => {
                if std::fs::exists(const_config.database.get_db_path()).unwrap() {
                    std::fs::remove_file(const_config.database.get_db_path()).expect("Failed to remove old database");
                }

                println!("Database removed");

                if recreate_only {
                    db::Database::new(const_config.database.get_db_path());
                    std::process::exit(0);
                }
            }
        }
    }

    let database = db::Database::new(const_config.database.get_db_path());
    let arc_db = Arc::new(Mutex::new(database));

    let ollama = match const_config.ollama.get_url() {
        Some(s) => Ollama::try_new(s).expect("Failed to create ollama instance"),
        None => Ollama::default()
    };

    let handler = Update::filter_message().endpoint(move |bot: Bot, msg: Message| {
        type Hyeta = Result<(), teloxide::RequestError>;

        let ollama = ollama.clone();
        let const_config = const_config.clone();
        let busy = busy.clone();
        let arc_db = arc_db.clone();

        async move {
            let dyn_config = dyn_config::DynConfig::get();

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

            if msg_parts.len() < 2 || msg_parts[0].to_lowercase() != const_config.telegram.get_callsign() {
                return Hyeta::Ok(());
            }

            tokio::spawn({
                let arc_db = arc_db.clone();
                let user_id = from.id.0;
                let username = from.username.clone();
                let from = from.clone();

                async move {
                    let db = arc_db.lock().await;

                    if !db.is_user_in_database(user_id).expect("Failed to check if user is already in database") {
                        println!("Pushing user {}({}) to database", user_id, username_to_string(&username));

                        db.add_user(from.clone()).expect("Failed to add user to the database");
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

            let prompt = format!("{} пишет: ", username_to_string(&from.username)) + &msg_parts[1..].join(" ")[..];
            println!("Request from {}({}): {}", from.first_name, username_to_string(&from.username), prompt);

            let req = GenerationRequest::new(dyn_config.ollama.get_model(), prompt)
                .think(dyn_config.ollama.get_thinking())
                .system(dyn_config.ollama.get_system_prompt());

            match ollama.generate(req).await {
                Ok(res) => {
                    let messages = res.response.split_by(const_config.telegram.get_message_length());

                    for message in messages {
                        bot.send_message(chat_id, message)
                            .reply_parameters(ReplyParameters::new(msg.id))
                            .await?;
                    }
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Ошибка генерации: {}", e))
                        .reply_parameters(ReplyParameters::new(msg.id))
                        .await?;
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

pub fn username_to_string(username: &Option<String>) -> String {
    match username {
        Some(s) => format!("@{s}"),
        None => "NO_USERNAME".to_string(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_print_test() {
        assert_eq!(&username_to_string(&Some("kaka".to_string())), "@kaka");
        assert_eq!(&username_to_string(&None), "NO_USERNAME");
    }
}