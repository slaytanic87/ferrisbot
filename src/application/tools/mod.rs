mod member_ban;
mod member_mute;
mod websearch;
pub use member_ban::KickUserParams;
pub use member_ban::KickUserWithoutBan;
pub use member_ban::KICK_USER_WITHOUTBAN;
pub use member_ban::KICK_USER_WITHOUTBAN_DESCRIPTION;
pub use member_mute::MuteMemberParams;
pub use member_mute::MuteMember;
pub use member_mute::MUTE_MEMBER;
pub use member_mute::MUTE_MEMBER_DESCRIPTION;
pub use websearch::WebSearch;
pub use websearch::WebSearchParams;
pub use websearch::WEB_SEARCH;
pub use websearch::WEB_SEARCH_DESCRIPTION;

use serde_json::Value;
use std::error::Error;

pub async fn execute_tool(
    tool_name: &str,
    parameters: Value,
) -> std::result::Result<String, Box<dyn Error + Sync + Send>> {
    match tool_name {
        WEB_SEARCH => {
            let mut websearch = WebSearch::new();
            websearch.execute(parameters).await
        }
        KICK_USER_WITHOUTBAN => {
            let kick_user = KickUserWithoutBan::new();
            kick_user.execute(parameters).await
        }
        MUTE_MEMBER => {
            let mute_member = MuteMember::new();
            mute_member.execute(parameters).await
        }
        _ => Err("Tool not found".into()),
    }
}
