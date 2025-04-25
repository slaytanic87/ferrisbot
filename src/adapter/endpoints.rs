use crate::{application::Color, Moderator};
use mobot::{Action, BotState, Event, State};
use regex::{Regex, RegexSet};


fn extract_username(value: &str) -> Vec<&str> {
    let separator = Regex::new(r"@.+");
    match separator {
        Ok(sep) => sep.split(value).collect(),
        Err(_) => Vec::new(),
    }
}

fn extract_coordinates(value: &str) -> Vec<&str> {
    let separator = Regex::new(r"#[A-H]{1}[0-7]{1}");
    match separator {
        Ok(sep) => sep.find_iter(value).map(|value| value.as_str()).collect(),
        Err(_) => Vec::new(),
    }
}

fn extract_color(value: &str) -> Option<Color> {
    let matcher_set = RegexSet::new([r"#white", r"#black"]);
    let matches: Vec<usize> = match matcher_set {
        Ok(set) => set
            .matches(value.to_lowercase().as_str())
            .into_iter()
            .collect(),
        Err(_) => panic!("Regex expression for extracting the color is invalid!"),
    };
    if matches.is_empty() {
        return None;
    }

    match matches[0] {
        0 => Some(Color::White),
        1 => Some(Color::Black),
        2_usize.. => None,
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
    let user_opt: Option<String> = event.update.get_message()?.clone().chat.username;
    let message: String = event.update.get_message()?.clone().text.unwrap().clone();
    let mut chess_controller = state.get().write().await;
    let reply_rs = chess_controller
        .moderator
        .chat(user_opt.clone().unwrap(), message)
        .await;
    if chess_controller.moderator.is_administrator(user_opt.unwrap().as_str()) {
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
    let mut chess_controller = state.get().write().await;
    let user_opt: Option<String> = event.update.get_message()?.clone().chat.username;
    let message: Option<String> = event.update.get_message()?.clone().text;

    if let None = message {
        return Ok(Action::ReplyText("User not found".into()));
    }
    if let None = user_opt {
        return Ok(Action::ReplyText("Admin username not found".into()));
    }
    if !chess_controller.moderator.is_administrator(user_opt.unwrap().as_str()) {
        return Ok(Action::ReplyText("You don't have permission to add".into()));
    }

    let message = message.unwrap();
    let extracted_usernames: Vec<&str> = extract_username(message.as_str());
    if extracted_usernames.is_empty() {
        return Ok(Action::ReplyText("Missing usernames".into()));
    }

    for user in extracted_usernames.iter() {
        user.to_string().remove(0);
        chess_controller.moderator.add_administrator(user.to_string());
    }
    Ok(Action::ReplyText("Added to admin list".into()))
}

