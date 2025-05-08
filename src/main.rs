use ferrisbot::BotController;
use mobot::{api::BotCommand, Client, Matcher, Route, Router};
use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    mobot::init_logger();
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
    let controller = BotController::new("Kate");
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
            Route::Message(Matcher::Any),
            ferrisbot::handle_chat_messages,
        );

    router.start().await;
}
