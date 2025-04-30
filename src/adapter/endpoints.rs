use std::time::SystemTime;

use crate::Moderator;
use mobot::{
    api::{ChatPermissions, RestrictChatMemberRequest},
    Action, BotState, Event, State,
};
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

pub async fn bot_chat_greeting(
    event: Event,
    _state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let user_opt: Option<String> = event.update.get_message()?.clone().chat.username;
    Ok(Action::ReplyText(format!(
        "Hello {} Im a bot that helps you manage your group. You can mute users and add admins",
        user_opt.unwrap().as_str()
    )))
}

pub async fn bot_chat_actions(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let user_opt: Option<String> = event.update.get_message()?.clone().chat.username;
    let message: String = event.update.get_message()?.clone().text.unwrap().clone();
    let mut bot_controller = state.get().write().await;
    let reply_rs = bot_controller
        .moderator
        .chat(user_opt.clone().unwrap(), message)
        .await;
    if bot_controller
        .moderator
        .is_administrator(user_opt.unwrap().as_str())
    {
        return Ok(Action::Done);
    }
    if let Ok(reply_message) = reply_rs {
        return Ok(Action::ReplyText(reply_message));
    }
    Ok(Action::Done)
}

pub async fn add_admin_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let mut bot_controller = state.get().write().await;
    let user_opt: Option<String> = event.update.get_message()?.clone().chat.username;
    let message: Option<String> = event.update.get_message()?.clone().text;

    if message.is_none() {
        return Ok(Action::ReplyText("User not found".into()));
    }
    if user_opt.is_none() {
        return Ok(Action::ReplyText("Admin username not found".into()));
    }
    if !bot_controller
        .moderator
        .is_administrator(user_opt.unwrap().as_str())
    {
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

pub async fn mute_user_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let user_opt: Option<String> = event.update.get_message()?.clone().chat.username;
    let user = event.update.get_message()?.clone().from;
    if user.is_none() {
        return Ok(Action::Done);
    }

    let bot_controller = state.get().write().await;
    if !bot_controller
        .moderator
        .is_administrator(user_opt.unwrap().as_str())
    {
        return Ok(Action::Done);
    }

    let mute_time_60s = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 60;

    let restrict_chat_req = RestrictChatMemberRequest {
        chat_id: event.update.get_message()?.clone().chat.id.to_string(),
        user_id: user.clone().unwrap().id,
        permissions: ChatPermissions {
            can_send_messages: Some(false),
            can_send_audios: Some(false),
            can_send_documents: Some(false),
            can_send_photos: Some(false),
            can_send_videos: Some(false),
            can_send_video_notes: None,
            can_send_voice_notes: None,
            can_send_polls: None,
            can_send_other_messages: None,
            can_add_web_page_previews: None,
            can_change_info: None,
            can_invite_users: None,
            can_pin_messages: None,
            can_manage_topics: None,
        },
        use_independent_chat_permissions: Some(false),
        until_date: Some(mute_time_60s as i64),
    };

    let success_muted = event.api.restrict_chat_member(&restrict_chat_req).await?;

    if !success_muted {
        return Ok(Action::ReplyText("Failed to mute user".into()));
    }

    Ok(Action::ReplyText(format!(
        "@{} You are muted now!",
        user.unwrap().username.unwrap()
    )))
}
