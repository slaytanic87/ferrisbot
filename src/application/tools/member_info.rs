use std::env;

use crate::UserManagement;
use log::debug;
use mobot::{
    api::{ChatMember, GetChatMemberRequest},
    Client,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};

pub const MEMBER_INFO: &str = "get_member_info";
pub const MEMBER_INFO_DESCRIPTION: &str = "Get information of a chat member.";

#[derive(Deserialize, JsonSchema)]
pub struct MemberInfoParam {
    #[schemars(description = "Unique identifier for the target chat or channel.")]
    pub chat_id: String,

    #[schemars(description = "The name of the user.")]
    pub name: String,
}

pub struct GetMember {
    telegram_api: mobot::api::API,
    user_management: UserManagement,
}

impl GetMember {
    pub fn new() -> Self {
        let client = Client::new(env::var("TELEGRAM_TOKEN").unwrap());
        Self {
            telegram_api: mobot::api::API::new(client),
            user_management: UserManagement::new(),
        }
    }

    pub async fn execute(
        &self,
        params: Value,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let parameters = serde_json::from_value::<MemberInfoParam>(params)?;
        let user_opt = self
            .user_management
            .get_user_by_first_name(&parameters.name);

        let user_id: i64 = if let Some((_, user)) = user_opt {
            user.user_id
        } else {
            debug!(
                "User not found in memory: firstname={}, chat_id={}",
                parameters.name, parameters.chat_id
            );
            return Ok("Could find user, because user has never contribute in the chat".into());
        };

        let mem_info: ChatMember = self
            .telegram_api
            .get_chat_member(&GetChatMemberRequest::new(parameters.chat_id, user_id))
            .await?;

        let current_member = if let Some(current_member) = mem_info.is_member {
            if current_member {
                "is part of this chat at the moment"
            } else {
                "is not part of this chat at the moment"
            }
        } else {
            "is part of the chat"
        };

        let status_member = match &mem_info.status.as_str() {
            &"creator" => "is the owner of this chat",
            &"administrator" => "is an administrator",
            &"member" => "normal member",
            &"restricted" => "member has been restricted",
            &"pending" => "is pending to be approved",
            &"left" => "has left the chat",
            &"kicked" => "was kicked from the chat",
            _ => "unknown member status",
        };

        let last_name = if let Some(last_name) = mem_info.user.last_name {
            last_name
        } else {
            "N/A".into()
        };

        Ok(json!({
            "member_status": status_member,
            "first_name": mem_info.user.first_name,
            "last_name": last_name,
            "username": mem_info.user.username,
            "current_chat_member": current_member,
        })
        .to_string())
    }
}
