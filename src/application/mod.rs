mod member;
mod moderator;
pub mod tools;
pub use moderator::Moderator;
pub use moderator::NO_ACTION;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageInput {
    pub channel: String,
    pub user_id: String,
    pub chat_id: String,
    pub user: String,
    pub message: String
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ModeratorFeedback {
    pub user_id: String,
    pub chat_id: String,
    pub moderator: String,
    pub message: String,
}
