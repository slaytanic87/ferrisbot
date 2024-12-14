use ferrisgram::ChessController;
use mobot::{Client, Matcher, Route, Router};
use std::env;

#[tokio::main]
async fn main() {
    mobot::init_logger();

    let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());
    let controller = ChessController::new();
    let mut router: mobot::Router<ChessController> = Router::new(client).with_state(controller);
    router
        .add_route(
            Route::ChannelPost(Matcher::Prefix("/".into())),
            ferrisgram::chess_command_handler,
        )
        .add_route(Route::Message(Matcher::Any), ferrisgram::chess_chat_actions);

    router.start().await;
}
