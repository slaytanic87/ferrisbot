use std::env;

use mobot::{api::UnbanChatMemberRequest, Client};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

pub const KICK_USER_WITHOUTBAN: &str = "kick_user_withoutban";
pub const KICK_USER_WITHOUTBAN_DESCRIPTION: &str = "Kicks a user from the telegram chat.";

#[derive(Deserialize, JsonSchema)]
pub struct KickUserParams {
    #[schemars(description = "The user ID to kick the user from.")]
    pub user_id: i64,

    #[schemars(description = "The chat ID to kick the user from.")]
    pub chat_id: i64,
}

pub struct KickUserWithoutBan {
    telegram_api: mobot::api::API,
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
        }
    }

    pub async fn execute(
        &self,
        params: Value,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let parameters = serde_json::from_value::<KickUserParams>(params)?;

        /*
         * According to telegram bot api using this method can also unban a user in a supergroup or channel.
         * But by default, this method guarantees that after the call the user is not a member of the chat, but will be able to join it.
         * So if the user is a member of the chat they will also be removed from the chat.
         */
        let success_rs = self
            .telegram_api
            .unban_chat_member(&UnbanChatMemberRequest::new(
                parameters.chat_id.to_string(),
                parameters.user_id,
                Some(false),
            ))
            .await?;
        if !success_rs {
            return Ok("Failed to kick the member".into());
        }
        Ok(format!(
            "Member {} has been kicked from chat {}",
            parameters.chat_id, parameters.user_id
        ))
    }
}
