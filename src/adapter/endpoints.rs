use std::env;

use crate::{
    application::{
        self,
        tools::{
            self, KICK_USER_WITHOUTBAN, KICK_USER_WITHOUTBAN_DESCRIPTION, MUTE_MEMBER,
            MUTE_MEMBER_DESCRIPTION, WEB_SEARCH, WEB_SEARCH_DESCRIPTION,
        },
        MessageInput, ModeratorMessage,
    },
    Assistant, Moderator, UserManagement,
};
use log::debug;
use mobot::{
    api::{
        ChatAction, ChatPermissions, GetChatAdministratorsRequest, GetChatRequest,
        RestrictChatMemberRequest, SendChatActionRequest, SendMessageRequest,
    },
    Action, BotState, Event, State,
};
use schemars::schema_for;
use serde_json::Value;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone, BotState, Default)]
pub struct BotController {
    moderator: Moderator,
    assistant: Assistant,
    user_management: UserManagement,
    name: String,
    bot_username: String,
}

impl BotController {
    pub fn new(
        name: &str,
        bot_username: &str,
        task_template: &str,
        tool_prompt_template: &str,
    ) -> Self {
        let moderator = Moderator::new(name, task_template);
        let mut assistant = Assistant::new(tool_prompt_template);
        assistant.add_tool(
            WEB_SEARCH.to_string(),
            WEB_SEARCH_DESCRIPTION.to_string(),
            schema_for!(tools::WebSearchParams),
        );
        assistant.add_tool(
            KICK_USER_WITHOUTBAN.to_string(),
            KICK_USER_WITHOUTBAN_DESCRIPTION.to_string(),
            schema_for!(tools::KickUserParams),
        );
        assistant.add_tool(
            MUTE_MEMBER.to_string(),
            MUTE_MEMBER_DESCRIPTION.to_string(),
            schema_for!(tools::MuteMemberParams),
        );

        let mut user_management = UserManagement::new();
        user_management.set_managed_chat_id(env::var("MANAGED_CHAT_ID)").ok());

        Self {
            moderator,
            assistant,
            user_management,
            name: name.into(),
            bot_username: bot_username.into(),
        }
    }
}

pub async fn inactive_users_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let username_opt: Option<String> = event.update.from_user()?.clone().username;
    let bot_controller = state.get().write().await;
    let months: u64 = 6;
    let months_in_secs: u64 = months * 4 * 7 * 24 * 60 * 60;
    let inactive_users = bot_controller
        .user_management
        .get_inactive_users_since(std::time::Duration::from_secs(months_in_secs));
    let chat_id: i64 = event.update.chat_id()?;

    let managed_chat_id: Option<String> = bot_controller.user_management.managed_chat_id.clone();
    if managed_chat_id.is_some() && managed_chat_id.unwrap() == chat_id.to_string() {
        debug!(
            "This Chat {} is managed and this command is not designed for that purpose",
            chat_id
        );
        return Ok(Action::Done);
    }

    if !bot_controller
        .user_management
        .is_administrator(username_opt.unwrap().as_str())
        || inactive_users.is_empty()
    {
        return Ok(Action::Done);
    }

    let mut message = String::new();
    for user in inactive_users {
        message.push_str(&format!(
            "User {} is inactive last {} months\n",
            user.username, months
        ));
    }
    event
        .api
        .send_message(&SendMessageRequest::new(event.update.chat_id()?, message))
        .await?;
    Ok(Action::Done)
}

pub async fn init_bot(event: Event, state: State<BotController>) -> Result<Action, anyhow::Error> {
    let mut bot_controller = state.get().write().await;
    let chat_type = event.update.get_message()?.clone().chat.chat_type;

    if chat_type == "private" {
        return Ok(Action::Done);
    }

    let message_date_unix = event.update.get_message()?.clone().date;
    let chat_id: &str = &event.update.get_message()?.chat.id.to_string();
    let admin_list = event
        .api
        .get_chat_administrators(&GetChatAdministratorsRequest::new(chat_id.to_string()))
        .await?;

    bot_controller.user_management.clear_administrators();

    admin_list.iter().for_each(|admin| {
        let username_opt: Option<String> = admin.user.username.clone();
        if let Some(username) = username_opt {
            if !bot_controller
                .user_management
                .is_administrator(username.as_str())
            {
                bot_controller
                    .user_management
                    .register_administrator(username);
            }
        }
    });

    let chat_full_info_list = event
        .api
        .get_chat(&GetChatRequest::new(chat_id.to_string()))
        .await?;

    if let Some(active_usernames) = chat_full_info_list.active_usernames {
        active_usernames.iter().for_each(|username: &String| {
            if !bot_controller.user_management.contains_user(username) {
                bot_controller
                    .user_management
                    .add_user(-1, username, "", message_date_unix as u64);
            }
        });
    }

    Ok(Action::Done)
}

pub async fn bot_greeting_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let username_opt: Option<String> = event.update.from_user()?.clone().username;
    let bot_controller = state.get().write().await;
    let message_thread_id_opt: Option<i64> = event.update.get_message()?.clone().message_thread_id;
    let chat_id: i64 = event.update.chat_id()?;

    if !bot_controller
        .user_management
        .is_administrator(username_opt.unwrap().as_str())
    {
        return Ok(Action::Done);
    }
    event
        .api
        .send_chat_action(&SendChatActionRequest {
            chat_id,
            message_thread_id: message_thread_id_opt,
            action: ChatAction::Typing,
        })
        .await?;

    let response_rs = bot_controller.moderator.introduce_moderator().await;
    if let Ok(response) = response_rs {
        let moderator_feedback: ModeratorMessage = serde_json::from_str(&response)?;
        if let Some(message_thread_id) = message_thread_id_opt {
            let message_re =
                &SendMessageRequest::new(event.update.chat_id()?, moderator_feedback.message)
                    .with_message_thread_id(message_thread_id);

            event.api.send_message(message_re).await?;
            return Ok(Action::Done);
        }
        return Ok(Action::ReplyText(moderator_feedback.message));
    }
    Ok(Action::Done)
}

pub async fn handle_chat_messages(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let user_id: i64 = event.update.from_user()?.clone().id;
    let username_opt: Option<String> = event.update.from_user()?.clone().username;
    let last_activity_unix_time: u64 = event.update.get_message()?.clone().date as u64;
    let reply_to_message_opt = event.update.get_message()?.clone().reply_to_message;
    let first_name: String = event.update.from_user()?.clone().first_name;
    let message: Option<String> = event.update.get_message()?.clone().text;
    let message_thread_id: Option<i64> = event.update.get_message()?.clone().message_thread_id;
    let mut bot_controller: RwLockWriteGuard<'_, BotController> = state.get().write().await;
    let chat_id: i64 = event.update.chat_id()?;

    // Only text message is supported
    if message.is_none() {
        return Ok(Action::Done);
    }

    let managed_chat_id: Option<String> = bot_controller.user_management.managed_chat_id.clone();
    if managed_chat_id.is_some() && managed_chat_id.unwrap() != chat_id.to_string() {
        debug!(
            "Chat {} is not registered as managed chat, so it's ignored",
            chat_id
        );
        return Ok(Action::Done);
    }

    let topic = if let Some(reply_to_message) = reply_to_message_opt.as_ref() {
        reply_to_message
            .get("forum_topic_created")
            .unwrap()
            .get("name")
            .unwrap()
            .as_str()
            .unwrap()
    } else {
        &event.update.get_message()?.clone().chat.title.unwrap()
    };

    let username: String = username_opt.unwrap_or(user_id.to_string());

    bot_controller.user_management.update_user_activity(
        &username,
        &first_name,
        user_id,
        last_activity_unix_time,
    );

    let role: &str = bot_controller.user_management.determine_user_role(username.as_str());

    let text_message = &message.unwrap().replace(
        format!("@{}", bot_controller.bot_username).as_str(),
        &bot_controller.name,
    );
    let input = MessageInput {
        channel: topic.to_string(),
        user_role: role.to_string(),
        user_id: user_id.to_string(),
        chat_id: chat_id.to_string(),
        user: first_name,
        message: text_message.to_string(),
    };
    let input_json_str = serde_json::to_string(&input)?;
    let reply_rs = bot_controller
        .moderator
        .chat_forum(input_json_str.as_str())
        .await;

    if let Ok(reply_message) = reply_rs {
        let message_str: ModeratorMessage = serde_json::from_str(&reply_message)?;
        if message_str.message.contains(application::NO_ACTION) {
            return Ok(Action::Done);
        }

        event
            .api
            .send_chat_action(&SendChatActionRequest {
                chat_id,
                message_thread_id,
                action: ChatAction::Typing,
            })
            .await?;

        if let Some(thread_id) = message_thread_id {
            let message_re = &SendMessageRequest::new(event.update.chat_id()?, message_str.message)
                .with_message_thread_id(thread_id);
            event.api.send_message(message_re).await?;
            return Ok(Action::Done);
        }
        return Ok(Action::ReplyText(message_str.message));
    }
    Ok(Action::Done)
}

pub async fn chat_summarize_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let bot_controller: RwLockReadGuard<'_, BotController> = state.get().read().await;
    let message_thread_id: Option<i64> = event.update.get_message()?.clone().message_thread_id;
    let reply_to_message_opt = event.update.get_message()?.clone().reply_to_message;
    let chat_id: i64 = event.update.chat_id()?;

    let topic = if let Some(reply_to_message) = reply_to_message_opt.as_ref() {
        reply_to_message
            .get("forum_topic_created")
            .unwrap()
            .get("name")
            .unwrap()
            .as_str()
            .unwrap()
    } else {
        &event.update.get_message()?.clone().chat.title.unwrap()
    };

    event
        .api
        .send_chat_action(&SendChatActionRequest {
            chat_id,
            message_thread_id,
            action: ChatAction::Typing,
        })
        .await?;

    let summarize_message_rs = bot_controller.moderator.summarize_chat(topic).await;
    if let Ok(summary) = summarize_message_rs {
        if let Some(thread_id) = message_thread_id {
            let message_re = &SendMessageRequest::new(event.update.chat_id()?, summary)
                .with_message_thread_id(thread_id);
            event.api.send_message(message_re).await?;
            return Ok(Action::Done);
        }
        return Ok(Action::ReplyText(summary));
    }
    Ok(Action::Done)
}

fn extract_user_id_chat_attribute(json: &Option<Value>) -> i64 {
    json.as_ref()
        .unwrap()
        .get("from")
        .unwrap()
        .get("id")
        .unwrap()
        .as_i64()
        .unwrap()
}

fn extract_username_chat_attribute(json: &Option<Value>) -> String {
    json.as_ref()
        .unwrap()
        .get("from")
        .unwrap()
        .get("username")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

pub async fn mute_user_action(
    event: Event,
    state: State<BotController>,
) -> Result<Action, anyhow::Error> {
    let user_opt: Option<String> = event.update.from_user()?.clone().username;
    let reply_to_message_opt = &event.update.get_message()?.clone().reply_to_message;
    let message_thread_id: Option<i64> = event.update.get_message()?.clone().message_thread_id;

    if reply_to_message_opt.is_none() {
        debug!("No reply to message object has been found");
        return Ok(Action::Done);
    }

    let user_id_be_muted: i64 = extract_user_id_chat_attribute(reply_to_message_opt);
    let username_be_muted: String = extract_username_chat_attribute(reply_to_message_opt);

    let bot_controller: RwLockReadGuard<'_, BotController> = state.get().read().await;
    let username: String = user_opt.unwrap_or("unknown".to_string());
    if !bot_controller
        .user_management
        .is_administrator(username.as_str())
    {
        debug!("User {} don't have admin rights to mute", username);
        return Ok(Action::Done);
    }

    if bot_controller
        .user_management
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

    let is_successful_muted = event.api.restrict_chat_member(&restrict_chat_req).await?;

    if !is_successful_muted {
        return Ok(Action::ReplyText("Failed to mute user".into()));
    }

    if let Some(thread_id) = message_thread_id {
        let message_re = &SendMessageRequest::new(
            event.update.chat_id()?,
            format!("@{} You are muted now!", username_be_muted),
        )
        .with_message_thread_id(thread_id);
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
    let reply_to_message_opt = &event.update.get_message()?.clone().reply_to_message;

    if reply_to_message_opt.is_none() {
        return Ok(Action::Done);
    }

    let user_id_be_unmuted: i64 = extract_user_id_chat_attribute(reply_to_message_opt);
    let username_be_unmuted: String = extract_username_chat_attribute(reply_to_message_opt);

    let bot_controller = state.get().write().await;
    if !bot_controller
        .user_management
        .is_administrator(user_opt.unwrap().as_str())
    {
        return Ok(Action::Done);
    }

    if bot_controller
        .user_management
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
    let is_successful_unmuted = event.api.restrict_chat_member(&restrict_chat_req).await?;
    if !is_successful_unmuted {
        return Ok(Action::ReplyText("Failed to unmute user".into()));
    }

    if let Some(thread_id) = message_thread_id {
        let message_re = &SendMessageRequest::new(
            event.update.chat_id()?,
            format!("@{} You are unmuted now!", username_be_unmuted),
        )
        .with_message_thread_id(thread_id);
        event.api.send_message(message_re).await?;
    }

    Ok(Action::Done)
}
