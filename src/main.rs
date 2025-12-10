use ferrisbot::{BotController, ASSISTANT_PROMPT_FILE, MODERATOR_PROMPT_FILE};
use mobot::{api::BotCommand, Client, Matcher, Route, Router};
use std::{env, fs::read_to_string};

fn read_prompt_template(path: &str) -> String {
    let template = read_to_string(path);
    match template {
        Ok(content) => content,
        Err(e) => {
            panic!("Failed to read the prompt template file: {}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    mobot::init_logger();
    let bot_name = env::var("BOT_NAME").unwrap_or_else(|_| "Ferrisbot".to_string());
    let bot_username = env::var("BOT_USERNAME").unwrap_or_else(|_| "FerrisModBot".to_string());
    let commands = vec![
        BotCommand {
            command: "greeting".into(),
            description: "Begrüß die Gruppe".into(),
        },
        BotCommand {
            command: "summary".into(),
            description: "Gib eine Zusammenfassung der letzten Chatverlauf".into(),
        },
    ];
    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());
    let controller = BotController::new(
        &bot_name,
        &bot_username,
        &read_prompt_template(MODERATOR_PROMPT_FILE),
        &read_prompt_template(ASSISTANT_PROMPT_FILE),
    );
    let mut router: mobot::Router<BotController> = Router::new(client).with_state(controller);

    router
        .api
        .set_my_commands(&mobot::api::SetMyCommandsRequest {
            commands,
            ..Default::default()
        })
        .await
        .unwrap();

    router
        .add_route(
            Route::Message(Matcher::BotCommand(String::from("init"))),
            ferrisbot::init_bot,
        )
        .add_route(
            Route::Message(Matcher::BotCommand(String::from("mute"))),
            ferrisbot::mute_user_action,
        )
        .add_route(
            Route::Message(Matcher::BotCommand(String::from("unmute"))),
            ferrisbot::unmute_user_action,
        )
        .add_route(
            Route::Message(Matcher::BotCommand(String::from("greeting"))),
            ferrisbot::bot_greeting_action,
        )
        .add_route(
            Route::Message(Matcher::BotCommand(String::from("summary"))),
            ferrisbot::chat_summarize_action,
        )
        .add_route(
            Route::Message(Matcher::BotCommand(String::from("inactiveUsers"))),
            ferrisbot::inactive_users_action,
        )
        // Matcher::Regex(format!("(?i)(@{bot_name}|@{bot_username})")
        .add_route(
            Route::Message(Matcher::Any),
            ferrisbot::handle_chat_messages,
        );

    router.start().await;
}
