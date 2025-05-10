use ferrisbot::BotController;
use mobot::{api::BotCommand, Client, Matcher, Route, Router};
use std::{env, fs::read_to_string};

fn read_prompt_template() -> String {
    let template = read_to_string("./role_definition.md");
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
    let bot_name = env::var("BOT_NAME").unwrap_or_else(|_| "Kate".to_string());
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
    let controller = BotController::new(&bot_name, &read_prompt_template());
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
            Route::Message(Matcher::BotCommand(String::from("admin"))),
            ferrisbot::add_admin_action,
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
            Route::Message(Matcher::Regex(String::from(format!("(?i)(@{bot_name})")))), 
            ferrisbot::web_search_action,
        )
        .add_route(
            Route::Message(Matcher::Any),
            ferrisbot::handle_chat_messages,
        );

    router.start().await;
}
