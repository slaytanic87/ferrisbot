use std::env;

use log::debug;
use mobot::{
    api::{BanChatMemberRequest, UnbanChatMemberRequest},
    Client,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::UserManagement;

pub const KICK_USER_WITHOUTBAN: &str = "kick_user_from_chat";
pub const KICK_USER_WITHOUTBAN_DESCRIPTION: &str = "Kicks a user from the telegram chat.";

#[derive(Deserialize, JsonSchema)]
pub struct KickUserParams {
    #[schemars(description = "The name of the user to be ban.")]
    pub name: String,

    #[schemars(description = "The chat ID to ban the user from.")]
    pub chat_id: i64,
}

pub struct KickUserWithoutBan {
    telegram_api: mobot::api::API,
    user_management: UserManagement,
}

impl Default for KickUserWithoutBan {
    fn default() -> Self {
        Self::new()
    }
}

impl KickUserWithoutBan {
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
        let parameters = serde_json::from_value::<KickUserParams>(params)?;

        let user_opt = self
            .user_management
            .get_user_by_first_name(&parameters.name);
        let user_id_be_ban: i64 = if let Some((_, user)) = user_opt {
            user.user_id
        } else {
            debug!(
                "User not found in memory: username={}, chat_id={}",
                parameters.name, parameters.chat_id
            );
            return Ok("Could not ban user, because user has never contribute in the chat".into());
        };

        let success_rs = self
            .telegram_api
            .ban_chat_member(&BanChatMemberRequest::new(
                parameters.chat_id.to_string(),
                user_id_be_ban,
                None,
                Some(false),
            ))
            .await?;
        if !success_rs {
            return Ok("Failed to kick the member".into());
        }
        Ok(format!(
            "Member {} has been kicked from chat {}",
            parameters.name, parameters.chat_id
        ))
    }
}
