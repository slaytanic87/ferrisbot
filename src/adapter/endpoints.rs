use crate::{application, Moderator};
use log::debug;
use mobot::{
    api::{ChatPermissions, GetChatAdministratorsRequest, RestrictChatMemberRequest, SendMessageRequest},
    Action, BotState, Event, State,
};
use regex::Regex;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

fn extract_username(value: &str) -> Vec<&str> {
    let separator = Regex::new(r"@.+");
    match separator {
        Ok(sep) => sep.split(value).collect(),
        Err(_) => Vec::new(),
    }
}

#[derive(Clone, BotState, Default)]
pub struct BotController {
    pub moderator: Moderator,
    pub name: String,
}

impl BotController {
    pub fn new(name: &str) -> Self {
        let moderator = Moderator::new(name);
        Self {
            moderator,
            name: name.into(),
        }
    }
}

pub async fn bot_greeting_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let mut bot_controller = state.get().write().await;
    let chat_type = event.update.get_message()?.clone().chat.chat_type;
    let message_thread_id_opt: Option<i64> = event.update.get_message()?.clone().message_thread_id;

    if chat_type != "private" {
        let chat_id = event.update.get_message()?.chat.id.to_string();
        let admin_list = event
            .api
            .get_chat_administrators(&GetChatAdministratorsRequest::new(chat_id))
            .await?;
        admin_list.iter().for_each(|admin| {
            let username_opt: Option<String> = admin.user.username.clone();
            if let Some(username) = username_opt {
                if !bot_controller.moderator.is_administrator(username.as_str()) {
                    bot_controller.moderator.register_administrator(username);
                }
            }
        });
    }
    let reponse_rs = bot_controller.moderator.introduce_moderator().await;
    if let Ok(response) = reponse_rs {
        if let Some(message_thread_id) = message_thread_id_opt {
            let message_re = &SendMessageRequest::new(event.update.chat_id()?, response).with_message_thread_id(message_thread_id);
            event.api.send_message(message_re).await?;
            return Ok(Action::Done);
        }
        return Ok(Action::ReplyText(response));
    }
    Ok(Action::Done)
}

pub async fn handle_chat_messages(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let username_opt: Option<String> = event.update.from_user()?.clone().username;
    let first_name: String = event.update.from_user()?.clone().first_name;
    let message: Option<String> = event.update.get_message()?.clone().text;
    let message_thread_id: Option<i64> = event.update.get_message()?.clone().message_thread_id;

    // Only text message is supported
    if message.is_none() {
        return Ok(Action::Done);
    }
    let channel_id: String = if let Some(thread_msg_id) = message_thread_id {
        thread_msg_id.to_string()
    } else {
        event.update.chat_id()?.to_string()
    };

    let mut bot_controller: RwLockWriteGuard<'_, BotController> = state.get().write().await;
    let username = username_opt.unwrap_or("unknown".to_string());
    if bot_controller
        .moderator
        .is_administrator(username.as_str())
    {
        debug!("Ignoring admin user {}", username);
        return Ok(Action::Done);
    }

    let reply_rs = bot_controller
        .moderator
        .chat_forum(channel_id, first_name, message.unwrap())
        .await;

    if let Ok(reply_message) = reply_rs {
        if reply_message.contains(application::NO_ACTION) {
            return Ok(Action::Done);
        }
        if let Some(thread_id) = message_thread_id {
            let message_re = &SendMessageRequest::new(event.update.chat_id()?, reply_message).with_message_thread_id(thread_id);
            event.api.send_message(message_re).await?;
            return Ok(Action::Done);
        }
        return Ok(Action::ReplyText(reply_message));
    }
    Ok(Action::Done)
}

pub async fn add_admin_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let mut bot_controller: RwLockWriteGuard<'_, BotController> = state.get().write().await;
    let user_opt: Option<String> = event.update.from_user()?.clone().username;
    let is_forum: Option<bool> = event.update.get_message()?.clone().chat.is_forum;
    let message: Option<String> = event.update.get_message()?.clone().text;

    if message.is_none() || user_opt.is_none() {
        return Ok(Action::ReplyText("User or Message not found".into()));
    }

    if is_forum.is_some() && is_forum.unwrap() {
        return Ok(Action::ReplyText(
            "Adding user to admin list is not allowed in topics".into(),
        ));
    }

    if !bot_controller
        .moderator
        .is_administrator(user_opt.unwrap_or("unknown".to_string()).as_str())
    {
        return Ok(Action::ReplyText(
            "You don't have permission to nominate users".into(),
        ));
    }

    let message = message.unwrap();
    let extracted_usernames: Vec<&str> = extract_username(message.as_str());
    if extracted_usernames.is_empty() {
        return Ok(Action::ReplyText("Missing usernames".into()));
    }
    debug!("usernames: {:?}", extracted_usernames);

    for user in extracted_usernames.iter() {
        user.to_string().remove(0);
        bot_controller.moderator.register_administrator(user.to_string());
    }
    Ok(Action::ReplyText("Added to admin list".into()))
}

pub async fn chat_summarize_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let bot_controller: RwLockReadGuard<'_, BotController> = state.get().read().await;
    let message_thread_id: Option<i64> = event.update.get_message()?.clone().message_thread_id;

    let channel_id: String = if let Some(thread_msg_id) = message_thread_id {
        thread_msg_id.to_string()
    } else {
        event.update.chat_id()?.to_string()
    };

    let summerize_message_rs = bot_controller
        .moderator
        .summerize_chat(channel_id)
        .await;
    if let Ok(summary) = summerize_message_rs {
        if let Some(thread_id) = message_thread_id {
            let message_re = &SendMessageRequest::new(event.update.chat_id()?, summary).with_message_thread_id(thread_id);
            event.api.send_message(message_re).await?;
            return Ok(Action::Done);
        }
        return Ok(Action::ReplyText(summary));
    }
    Ok(Action::Done)
}

pub async fn mute_user_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let user_opt: Option<String> = event.update.from_user()?.clone().username;
    let reply_to_message_opt = event.update.get_message()?.clone().reply_to_message;
    let message_thread_id: Option<i64> = event.update.get_message()?.clone().message_thread_id;

    if reply_to_message_opt.is_none() {
        debug!("No reply to message object has beend found");
        return Ok(Action::Done);
    }

    let user_id_be_muted: i64 = reply_to_message_opt
        .as_ref()
        .unwrap()
        .get("from")
        .unwrap()
        .get("id")
        .unwrap()
        .as_i64()
        .unwrap();
    let username_be_muted: String = reply_to_message_opt
        .unwrap()
        .get("from")
        .unwrap()
        .get("username")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let bot_controller: RwLockReadGuard<'_, BotController> = state.get().read().await;
    let username: String = user_opt.unwrap_or("unknown".to_string());
    if !bot_controller
        .moderator
        .is_administrator(username.as_str())
    {
        debug!("User {} don't have admin rights to mute", username);
        return Ok(Action::Done);
    }

    if bot_controller
        .moderator
        .is_administrator(username_be_muted.as_str())
    {
        debug!("User {} is admin, can't mute", username_be_muted);
        return Ok(Action::Done);
    }

    let mute_time_60s: i64 = event.update.get_message()?.clone().date + 60;

    let restrict_chat_req = RestrictChatMemberRequest {
        chat_id: event.update.get_message()?.clone().chat.id.to_string(),
        user_id: user_id_be_muted,
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
        until_date: Some(mute_time_60s),
    };

    let is_success_muted = event.api.restrict_chat_member(&restrict_chat_req).await?;

    if !is_success_muted {
        return Ok(Action::ReplyText("Failed to mute user".into()));
    }

    if let Some(thread_id) = message_thread_id {
        let message_re = &SendMessageRequest::new(event.update.chat_id()?, format!(
            "@{} You are muted now!",
            username_be_muted
        )).with_message_thread_id(thread_id);
        event.api.send_message(message_re).await?;
    }

    Ok(Action::Done)
}

pub async fn unmute_user_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let user_opt: Option<String> = event.update.from_user()?.clone().username;
    let message_thread_id: Option<i64> = event.update.get_message()?.clone().message_thread_id;

    let reply_to_message_opt = event.update.get_message()?.clone().reply_to_message;

    if reply_to_message_opt.is_none() {
        return Ok(Action::Done);
    }

    let user_id_be_unmuted: i64 = reply_to_message_opt
        .as_ref()
        .unwrap()
        .get("from")
        .unwrap()
        .get("id")
        .unwrap()
        .as_i64()
        .unwrap();

    let username_be_unmuted: String = reply_to_message_opt
        .unwrap()
        .get("from")
        .unwrap()
        .get("username")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let bot_controller = state.get().write().await;
    if !bot_controller
        .moderator
        .is_administrator(user_opt.unwrap().as_str())
    {
        return Ok(Action::Done);
    }

    if bot_controller
        .moderator
        .is_administrator(username_be_unmuted.as_str())
    {
        debug!("User {} is admin, can't unmute", username_be_unmuted);
        return Ok(Action::Done);
    }

    let restrict_chat_req = RestrictChatMemberRequest {
        chat_id: event.update.get_message()?.clone().chat.id.to_string(),
        user_id: user_id_be_unmuted,
        permissions: ChatPermissions {
            can_send_messages: Some(true),
            can_send_audios: Some(true),
            can_send_documents: Some(true),
            can_send_photos: Some(true),
            can_send_videos: Some(true),
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
        until_date: None,
    };
    let success_unmuted = event.api.restrict_chat_member(&restrict_chat_req).await?;
    if !success_unmuted {
        return Ok(Action::ReplyText("Failed to unmute user".into()));
    }

    if let Some(thread_id) = message_thread_id {
        let message_re = &SendMessageRequest::new(event.update.chat_id()?, format!(
            "@{} You are unmuted now!",
            username_be_unmuted
        )).with_message_thread_id(thread_id);
        event.api.send_message(message_re).await?;
    }

    Ok(Action::Done)
}
