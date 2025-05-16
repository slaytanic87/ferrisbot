use std::{env, time::SystemTime};

use mobot::{
    api::{ChatPermissions, RestrictChatMemberRequest},
    Client,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

pub const MUTE_MEMBER: &str = "mute_member";
pub const MUTE_MEMBER_DESCRIPTION: &str = "Mute a member from the telegram chat.";

#[derive(Deserialize, JsonSchema)]
pub struct MuteMemberParams {
    #[schemars(description = "The user ID to mute the user from.")]
    pub user_id: i64,

    #[schemars(description = "The chat ID to mute the user from.")]
    pub chat_id: i64,

    #[schemars(description = "Mute time in seconds.")]
    pub mute_time: i64,
}

pub struct MuteMember {
    telegram_api: mobot::api::API,
}
impl Default for MuteMember {
    fn default() -> Self {
        Self::new()
    }
}

impl MuteMember {
    pub fn new() -> Self {
        let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());
        Self {
            telegram_api: mobot::api::API::new(client),
        }
    }

    pub async fn execute(
        &self,
        params: Value,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let parameters = serde_json::from_value::<MuteMemberParams>(params)?;
        let user_id_be_muted = parameters.user_id;
        let chat_id = parameters.chat_id;
        let mute_time_seconds: i64 = parameters.mute_time + SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64;
        let restrict_chat_req = RestrictChatMemberRequest {
            chat_id: chat_id.to_string(),
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
            until_date: Some(mute_time_seconds),
        };
        let is_successful_muted = self.telegram_api.restrict_chat_member(&restrict_chat_req).await?;
        if !is_successful_muted {
            return Ok("Failed to mute the member".into());
        }
        Ok("Member muted successfully".into())
    }
}
