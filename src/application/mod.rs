mod moderator;
pub mod tools;
pub use moderator::Moderator;
pub use moderator::NO_ACTION;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageInput {
    pub channel: String,
    pub user: String,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ModeratorFeedback {
    pub moderator: String,
    pub message: String,
}
