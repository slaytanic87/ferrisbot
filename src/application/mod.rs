mod assistant_agent;
mod member;
mod moderator_agent;
pub mod tools;
pub use assistant_agent::Assistant;
pub use member::UserManagement;
pub use moderator_agent::Moderator;
pub use moderator_agent::NO_ACTION;
pub use moderator_agent::MODERATOR_PROMPT_FILE;
pub use assistant_agent::ASSISTANT_PROMPT_FILE;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageInput {
    pub channel: String,
    pub user_id: String,
    pub chat_id: String,
    pub user: String,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ModeratorFeedback {
    pub user_id: String,
    pub chat_id: String,
    pub moderator: String,
    pub message: String,
}
