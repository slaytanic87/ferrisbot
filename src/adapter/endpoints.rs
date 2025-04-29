use crate::Moderator;
use mobot::{Action, BotState, Event, State};
use regex::Regex;


fn extract_username(value: &str) -> Vec<&str> {
    let separator = Regex::new(r"@.+");
    match separator {
        Ok(sep) => sep.split(value).collect(),
        Err(_) => Vec::new(),
    }
}

#[derive(Clone, BotState)]
pub struct BotController {
    pub moderator: Moderator,
}

impl Default for BotController {
    fn default() -> Self {
        Self::new()
    }
}

impl BotController {
    pub fn new() -> Self {
        let moderator = Moderator::new();
        Self { moderator }
    }
}

pub async fn bot_chat_actions(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let user_opt: Option<String> = event.update.get_post()?.clone().chat.username;
    let message: String = event.update.get_post()?.clone().text.unwrap().clone();
    let mut bot_controller = state.get().write().await;
    let reply_rs = bot_controller
        .moderator
        .chat(user_opt.clone().unwrap(), message)
        .await;
    if bot_controller.moderator.is_administrator(user_opt.unwrap().as_str()) {
        return Ok(Action::Done);
    }
    if let Ok(reply_message) = reply_rs {
        return Ok(Action::ReplyMarkdown(reply_message));
    }
    Ok(Action::Done)
}

pub async fn add_admin_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let mut bot_controller = state.get().write().await;
    let user_opt: Option<String> = event.update.get_post()?.clone().chat.username;
    let message: Option<String> = event.update.get_post()?.clone().text;

    if message.is_none() {
        return Ok(Action::ReplyText("User not found".into()));
    }
    if user_opt.is_none() {
        return Ok(Action::ReplyText("Admin username not found".into()));
    }
    if !bot_controller.moderator.is_administrator(user_opt.unwrap().as_str()) {
        return Ok(Action::ReplyText("You don't have permission to add".into()));
    }

    let message = message.unwrap();
    let extracted_usernames: Vec<&str> = extract_username(message.as_str());
    if extracted_usernames.is_empty() {
        return Ok(Action::ReplyText("Missing usernames".into()));
    }

    for user in extracted_usernames.iter() {
        user.to_string().remove(0);
        bot_controller.moderator.add_administrator(user.to_string());
    }
    Ok(Action::ReplyText("Added to admin list".into()))
}

