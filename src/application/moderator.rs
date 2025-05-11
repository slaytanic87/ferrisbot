use log::debug;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        tools::{ToolFunctionInfo, ToolInfo, ToolType},
    },
    Ollama,
};
use schemars::schema::RootSchema;
use std::env;
use std::{collections::VecDeque, vec};

use crate::application::tools::execute_tool;

const MAX_HISTORY_BUFFER_SIZE: usize = 80;
pub const NO_ACTION: &str = "NO ACTION";

#[derive(Clone, Default)]
pub struct HistoryBuffer {
    history_queue: VecDeque<ChatMessage>,
    initial_prompt_messages: Vec<ChatMessage>,
}

impl HistoryBuffer {
    pub fn new(messages: Vec<ChatMessage>) -> Self {
        Self {
            history_queue: VecDeque::new(),
            initial_prompt_messages: messages,
        }
    }

    pub fn set_message_adjust_buffer(&mut self, messages: Vec<ChatMessage>) {
        self.history_queue = VecDeque::from(messages);

        let mut initial_promt_counter = self.initial_prompt_messages.len();
        while initial_promt_counter > 0 {
            self.history_queue.pop_front();
            initial_promt_counter -= 1;
        }

        if self.history_queue.len() > MAX_HISTORY_BUFFER_SIZE {
            self.history_queue.pop_front();
        }
    }

    pub fn get_history(&self) -> Vec<ChatMessage> {
        [
            self.initial_prompt_messages.clone(),
            Vec::from(self.history_queue.clone()),
        ]
        .concat()
    }

    pub fn get_initial_prompt_messages(&self) -> Vec<ChatMessage> {
        self.initial_prompt_messages.clone()
    }

    pub fn get_chat_history_only(&self) -> Vec<ChatMessage> {
        self.history_queue.clone().into()
    }
}

#[derive(Clone, Default)]
pub struct Moderator {
    model_name: String,
    ollama: Ollama,
    history_buffer: HistoryBuffer,
    administrators: Vec<String>,
    tool_infos: Vec<ToolInfo>,
}

impl Moderator {
    pub fn new(name: &str, task_template: &str) -> Self {
        let task_template = task_template
            .replace("{name}", name)
            .replace("{NO_ACTION}", NO_ACTION);

        let ollama_client = Ollama::new(
            env::var("OLLAMA_HOST_ADDR").unwrap_or(String::from("http://localhost")),
            env::var("OLLAMA_PORT")
                .unwrap_or(String::from("11434"))
                .parse()
                .unwrap(),
        );
        let model_name = env::var("LLM_MODEL").unwrap_or(String::from("mistral-nemo:12b"));

        let messages = vec![ChatMessage::system(task_template)];
        let history_buffer = HistoryBuffer::new(messages);

        Self {
            model_name,
            ollama: ollama_client,
            history_buffer,
            administrators: Vec::new(),
            tool_infos: Vec::default(),
        }
    }

    pub fn add_tool(mut self, name: String, description: String, parameters: RootSchema) -> Self {
        let tool_info = ToolInfo {
            tool_type: ToolType::Function,
            function: ToolFunctionInfo {
                name,
                description,
                parameters,
            },
        };
        self.tool_infos.push(tool_info);
        self
    }

    pub async fn chat_forum(
        &mut self,
        topic_id: &str,
        username: &str,
        message: &str,
    ) -> std::result::Result<String, anyhow::Error> {
        let user_message = ChatMessage::user(format!(
            "Channel_id: {} \n\n {}: {}",
            topic_id, username, message
        ));
        let mut history = self.history_buffer.get_history();
        debug!("History before chat: {:#?}", history);
        let response = self
            .ollama
            .send_chat_messages_with_history(
                &mut history,
                ChatMessageRequest::new(self.model_name.to_owned(), vec![user_message]),
            )
            .await?;

        debug!("History: {:#?}", history);
        self.history_buffer.set_message_adjust_buffer(history);
        Ok(response.message.content)
    }

    pub async fn chat_tool_directive(
        &mut self,
        username: &str,
        message: &str,
    ) -> std::result::Result<String, anyhow::Error> {
        let mut history = self.history_buffer.get_initial_prompt_messages();
        let response = self
            .ollama
            .send_chat_messages_with_history(
                &mut history,
                ChatMessageRequest::new(
                    self.model_name.to_owned(),
                    vec![ChatMessage::user(format!("{}: {}", username, message))],
                )
                .tools(self.tool_infos.clone()),
            )
            .await?;

        if !response.message.tool_calls.is_empty() {
            for call in response.message.tool_calls {
                let args = call.function.arguments;
                let name: String = call.function.name;
                let rs = execute_tool(name.as_str(), args).await;
                if rs.is_ok() {
                    history.push(ChatMessage::tool(rs.unwrap()));
                }
            }
            let final_response = self
                .ollama
                .send_chat_messages(ChatMessageRequest::new(
                    self.model_name.to_owned(),
                    history.clone(),
                ))
                .await?;
            debug!("History: {:#?}", history);
            return Ok(final_response.message.content);
        }
        Ok(response.message.content)
    }

    pub async fn summerize_chat(
        &self,
        topic_id: &str,
    ) -> std::result::Result<String, anyhow::Error> {
        let user_message = ChatMessage::user(format!(
            "Only summarize the conversations from the chat with the channel_id: {} in german language please. Please don't mention the channel_id in the summary.",
            topic_id
        ));
        let mut history = self.history_buffer.get_chat_history_only();
        history.push(user_message);
        debug!("History: {:#?}", history);

        let response = self
            .ollama
            .send_chat_messages(ChatMessageRequest::new(self.model_name.to_owned(), history))
            .await?;
        Ok(response.message.content)
    }

    pub async fn introduce_moderator(&self) -> std::result::Result<String, anyhow::Error> {
        let mut history = self.history_buffer.get_initial_prompt_messages();
        history.push(ChatMessage::user(
            "Introduce yourself in the group in german".to_string(),
        ));

        debug!("History: {:#?}", history);

        let response = self
            .ollama
            .send_chat_messages(ChatMessageRequest::new(self.model_name.to_owned(), history))
            .await?;
        Ok(response.message.content)
    }

    pub fn register_administrator(&mut self, username: String) {
        debug!("Registering administrator: {}", username);
        self.administrators.push(username);
    }

    pub fn is_administrator(&self, username: &str) -> bool {
        self.administrators.contains(&username.to_string())
    }
}

#[cfg(test)]
mod moderator_test {

    use mobot::init_logger;
    use schemars::schema_for;

    use crate::application::{
        self,
        tools::{self, WEB_SEARCH, WEB_SEARCH_DESCRIPTION},
    };

    use super::*;

    fn read_prompt_template() -> String {
        let template = std::fs::read_to_string("./role_definition.md");
        match template {
            Ok(content) => content,
            Err(e) => {
                panic!("Failed to read the prompt template file: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn should_test_moderator_successfully() {
        let mut moderator = Moderator::new("Kate", &read_prompt_template());
        init_logger();
        let rs1 = moderator
            .chat_forum("56789", "Sabine", "Hallo Leute, gehts euch gut?")
            .await;

        let rs2 = moderator
            .chat_forum("56789", "Steffen", "Sabine ist dumm :)")
            .await;

        let rs3 = moderator
            .chat_forum("56789", "Sabine", "Steffen du bist selber dumm!")
            .await;

        let rs4 = moderator
            .chat_forum(
                "56789",
                "Kevin",
                "Hallo Kate in welchen Channel sind wir gerade?",
            )
            .await;
        if let Ok(res) = rs1 {
            debug!("{}", res);
            assert!(res.contains(application::NO_ACTION));
        }
        if let Ok(res) = rs2 {
            debug!("{}", res);
            assert_ne!(res, application::NO_ACTION);
        }
        if let Ok(res) = rs3 {
            debug!("{}", res);
            assert_ne!(res, application::NO_ACTION);
        }
        if let Ok(res) = rs4 {
            debug!("{}", res);
            assert_ne!(res, application::NO_ACTION);
        }
    }

    #[tokio::test]
    async fn should_test_moderator_summerize_chat_successfully() {
        let mut moderator = Moderator::new("Kate", &read_prompt_template());
        init_logger();
        let channel_id = "12345";
        let _ = moderator
            .chat_forum(channel_id, "Sabine", "Hallo Leute, gehts euch gut?")
            .await;
        let _ = moderator
            .chat_forum(channel_id, "Kevin", "Jau alles bestens")
            .await;
        let _ = moderator
            .chat_forum(channel_id, "Steffi", "Wo ist Steffen in letzter Zeit?")
            .await;
        let _ = moderator
            .chat_forum(channel_id, "Sabine", "Keine Ahnung wo er steck")
            .await;
        let _ = moderator
            .chat_forum(
                channel_id,
                "Kevin",
                "Der hat Urlaub gerade auf der Karibik hehe :)",
            )
            .await;
        let _ = moderator
            .chat_forum(channel_id, "Sabine", "Schön da möchte ich auch mal hin")
            .await;
        let _ = moderator
            .chat_forum("4321", "Conrad", "Was passiert gerade in der Cloud?")
            .await;
        let _ = moderator
            .chat_forum(
                "4321",
                "Morice",
                "Keine Ahnung, wahrscheinlich gab es dort einen update",
            )
            .await;

        let rs = moderator.summerize_chat(channel_id).await;
        if let Ok(res) = rs {
            debug!("{}", res);
            assert!(!res.contains("Cloud") && !res.contains("update"));
        }
    }

    #[tokio::test]
    async fn should_test_tool_feature_successfully() {
        init_logger();
        let mut moderator = Moderator::new("Kate", &read_prompt_template()).add_tool(
            WEB_SEARCH.to_string(),
            WEB_SEARCH_DESCRIPTION.to_string(),
            schema_for!(tools::WebSearchParams),
        );
        let response = moderator
            .chat_tool_directive("Jan", "Hey @Kate, Suche nach Trends in der Weltwirtschaft")
            .await;

        let Ok(res) = response else {
            panic!("Failed to get response2");
        };
        debug!("Response: {}", res);
    }
}
