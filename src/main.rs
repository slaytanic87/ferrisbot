use ferrisbot::BotController;
use mobot::{api::BotCommand, Client, Matcher, Route, Router};
use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    mobot::init_logger();
    let commands = vec![
        BotCommand {
            command: "admin".into(),
            description: "Add a user to admin list".into(),
        },
        BotCommand {
            command: "greeting".into(),
            description: "Greet the user".into(),
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
            ferrisbot::mute_user_action,
        )
        .add_route(
            Route::Message(Matcher::BotCommand(String::from("greeting"))),
            ferrisbot::bot_chat_greeting,
        )
        .add_route(Route::Message(Matcher::Any), |event, state| {
            ferrisbot::bot_chat_actions(event, state)
        });

    router.start().await;
}
