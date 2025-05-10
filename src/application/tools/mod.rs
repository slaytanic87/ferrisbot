mod websearch;
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
        _ => Err("Tool not found".into()),
    }
}
