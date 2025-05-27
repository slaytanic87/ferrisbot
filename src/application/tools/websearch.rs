use json2markdown::MarkdownRenderer;
use log::info;
use reqwest::Client;
use schemars::JsonSchema;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

pub const WEB_SEARCH: &str = "web_search";
pub const WEB_SEARCH_DESCRIPTION: &str = "Searches the web using DuckDuckGo's HTML interface.";

#[derive(Deserialize, JsonSchema)]
pub struct WebSearchParams {
    #[schemars(description = "The search query to send to DuckDuckGo.")]
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    title: String,
    content: String,
}

pub struct WebSearch {
    pub web_url: String,
    pub client: Client,
}

impl Default for WebSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSearch {
    pub fn new() -> Self {
        Self {
            web_url: "https://duckduckgo.com".to_string(),
            client: Client::new(),
        }
    }

    async fn search(&self, query: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
        let url = format!("{}/html/?q={}", self.web_url, query);
        info!("Searching... : {}", url);
        let response = self.client.get(&url).send().await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);

        let result_selector = Selector::parse(".web-result").unwrap();
        let title_selector = Selector::parse(".result__a").unwrap();
        let snippet_selector = Selector::parse(".result__snippet").unwrap();
        let search_results = document
            .select(&result_selector)
            .map(|result| {
                let title = result
                    .select(&title_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("");
                let snippet = result
                    .select(&snippet_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("");
                SearchResult {
                    title,
                    content: snippet,
                }
            })
            .collect::<Vec<_>>();

        let renderer = MarkdownRenderer::default();
        let json_result = serde_json::json!(search_results);

        Ok(renderer.render(&json_result))
    }

    pub async fn execute(
        &mut self,
        params: Value,
    ) -> std::result::Result<String, Box<dyn Error + Sync + Send>> {
        let param = serde_json::from_value::<WebSearchParams>(params)?;
        let result_html = self.search(&param.query).await?;
        Ok(result_html)
    }
}
