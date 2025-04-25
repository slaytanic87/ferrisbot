use crate::{application::Color, Moderator};
use anyhow::{anyhow, Error};
use mobot::{Action, BotState, Event, State};
use regex::{Regex, RegexSet};

fn extract_cmds(value: &str) -> Vec<&str> {
    let separator = Regex::new(r"#[a-z]+");
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

pub async fn chess_command_handler(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let command: &String = event
        .update
        .get_message()?
        .text
        .as_ref()
        .ok_or(anyhow!("Not a command"))?;
    let mut chess_controller = state.get().write().await;

    let response: String = match command.as_str() {
        "/chessgame" => {
            format!("{} \n {}", "board", "New game!".to_owned())
        }
        "/help" => {
            let introduction =
                "This is an asyncron chat based chess game. Type /newgame to restart the game"
                    .to_owned();
            let instruction =
                "To make a move: #move #(white or black) #(start coordinate) #(end coordinate) \n
                          e.g #move #white #A0 #A1";
            format!("{} \n {}", introduction, instruction)
        }
        _ => "Unknown command".to_owned(),
    };

    Ok(Action::ReplyText(response))
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
        .chat(user_opt.unwrap(), message)
        .await;
    if let Ok(reply_message) = reply_rs {
        return Ok(Action::ReplyMarkdown(reply_message));
    }
    Ok(Action::Done)
}

pub async fn add_admin_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    Ok(Action::Done)
}


pub async fn chess_chat_actions(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let message: String = event.update.get_message()?.clone().text.unwrap().clone();
    let cmds: Vec<&str> = extract_cmds(message.as_str());
    if cmds.is_empty() {
        return Ok(Action::Done);
    }
    let command: &&str = cmds.first().unwrap();
    match *command {
        "#move" => {
            let coordinates: Vec<&str> = extract_coordinates(message.as_str());
            if coordinates.len() <= 1 {
                return Ok(Action::ReplyText(
                    "missing start and target coordinates for a step".into(),
                ));
            }
            let player_color: Option<Color> = extract_color(&message);
            if player_color.is_none() {
                return Ok(Action::ReplyMarkdown(
                    "Unknown or missing player only #white or #black is allowed".to_owned(),
                ));
            }

            Ok(Action::ReplyMarkdown(format!(
                "{} \n {}",
                "board", "move_msg"
            )))
        }
        _ => Ok(Action::Done),
    }
}
