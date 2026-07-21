use std::env;

use log::debug;
use mobot::{
    api::{MessageReactionRequest, ReactionType},
    Client,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

pub const MESSAGE_REACTION: &str = "message_reaction";
pub const MESSAGE_REACTION_DESCRIPTION: &str = "send an emoji on a message of an user.";

#[derive(Deserialize, JsonSchema)]
pub struct MessageReactionParam {
    #[schemars(description = "Unique identifier for the target chat or channel.")]
    pub chat_id: String,
    #[schemars(description = "Identifier of a message to react on.")]
    pub message_id: i64,
    #[schemars(description = "Choosen emoji to react on a message.")]
    pub emoji: String,
}

pub struct MessageReaction {
    telegram_api: mobot::api::API,
}

impl MessageReaction {
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
        let parameters = serde_json::from_value::<MessageReactionParam>(params)?;

        let reaction_result = self
            .telegram_api
            .set_message_reaction(&MessageReactionRequest::new(
                parameters.chat_id.parse::<i64>().unwrap(),
                parameters.message_id,
                Some(vec![ReactionType::new(
                    "emoji".to_string(),
                    Some(parameters.emoji.clone()),
                    None,
                )]),
                None,
            ))
            .await?;

        if reaction_result {
            debug!(
                "Reaction done for message id {} with {}",
                parameters.message_id, parameters.emoji
            );
            Ok("done".to_string())
        } else {
            log::error!(
                "Could not react on message id {} with {}",
                parameters.message_id,
                parameters.emoji
            );
            Ok("Could not react on message".to_string())
        }
    }
}
