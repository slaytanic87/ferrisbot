mod adapter;
mod application;
pub use adapter::bot_chat_actions;
pub use adapter::chess_chat_actions;
pub use adapter::chess_command_handler;
pub use adapter::add_admin_action;
pub use adapter::BotController;
pub use application::Moderator;

/* 
use mobot::BotRequest;
use mobot::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetChatPermissionRequest {
    pub chat_id: String,
    pub permissions: ChatPermissions,
    pub use_independent_chat_permissions: Option<bool>
}

#[derive(Debug, Clone, Serialize, Deserialize, BotRequest)]
pub struct ChatPermissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send_messages: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send_audios: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send_documents: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send_photos: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send_videos: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send_video_notes: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send_voice_notes: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send_polls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send_other_messages: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_add_web_page_previews: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_info: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_invite_users: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_pin_messages: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_manage_topics: Option<bool>
}

pub struct API {
    /// The underlying HTTP client.
    pub client: Client,
}

impl API {
    pub async fn set_chat_permissions(&self, req: &SetChatPermissionRequest) -> anyhow::Result<bool> {
        self.client.post("setChatPermissions", req).await
    }
}

*/