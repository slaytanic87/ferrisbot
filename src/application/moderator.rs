use log::debug;
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use std::{collections::VecDeque, env, error::Error, vec};

const MAX_HISTORY_BUFFER_SIZE: usize = 15;

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
}

#[derive(Clone, Default)]
pub struct Moderator {
    model_name: String,
    ollama: Ollama,
    history_buffer: HistoryBuffer,
    administrators: Vec<String>,
}

impl Moderator {
    pub fn new(name: &str) -> Self {
        let ollama_client = Ollama::new(
            env::var("OLLAMA_HOST_ADDR").unwrap_or(String::from("http://localhost")),
            env::var("OLLAMA_PORT")
                .unwrap_or(String::from("11434"))
                .parse()
                .unwrap(),
        );
        let model_name = env::var("LLM_MODEL").unwrap_or(String::from("llama3.2:latest"));

        let messages = vec![ChatMessage::system(format!(
            "As an AI assistant in a german speaking Telegram group, your name is {} and your role is supporting the admins as a moderator to prevent members using vulgar expression, hot discussions or insult each other.
Remember that your task is just observe the discussions in the group and keep it peacefully, otherwise you have to advise the members in the chat group directly to follow the rules if they are not following the rules repeatedly. The preferred language in the chat group is German.
You just need to repond with the chat group member if you see a user using vulgar expression or insulting each other. Otherwise just answer with: [NO ACTION]",
            name
        ))];
        let history_buffer = HistoryBuffer::new(messages);

        Self {
            model_name,
            ollama: ollama_client,
            history_buffer,
            administrators: Vec::new(),
        }
    }

    pub async fn chat(
        &mut self,
        username: String,
        message: String,
    ) -> Result<String, Box<dyn Error>> {
        let user_message = ChatMessage::user(format!("User: {}, Message: {}", username, message));
        let mut history = self.history_buffer.get_history();
        debug!("History before chat: {:#?}", history);
        let response = self
            .ollama
            .send_chat_messages_with_history(
                &mut history,
                ChatMessageRequest::new(self.model_name.to_owned(), vec![user_message]),
            )
            .await?;
        debug!("History after chat: {:#?}", history);
        self.history_buffer.set_message_adjust_buffer(history);
        Ok(response.message.content)
    }

    pub fn add_administrator(&mut self, username: String) {
        self.administrators.push(username);
    }

    pub fn is_administrator(&self, username: &str) -> bool {
        self.administrators.contains(&username.to_string())
    }
}

#[cfg(test)]
mod moderator_test {

    use mobot::init_logger;

    use super::*;

    #[tokio::test]
    async fn should_test_moderator_successfully() {
        let mut moderator = Moderator::new("Kate");
        init_logger();
        let rs1 = moderator
            .chat(
                "Fuffi".to_string(),
                "Hallo Leute, gehts euch gut?".to_string(),
            )
            .await;

        let rs2 = moderator
            .chat("Steffen".to_string(), "Fuffi ist dumm :)".to_string())
            .await;

        let rs3 = moderator
            .chat(
                "Fuffi".to_string(),
                "Steffen du bist selber dumm!".to_string(),
            )
            .await;

        if let Ok(res) = rs1 {
            debug!("{}", res);
            assert_eq!(res, "[NO ACTION]");
        }
        if let Ok(res) = rs2 {
            debug!("{}", res);
            assert_ne!(res, "");
        }
        if let Ok(res) = rs3 {
            debug!("{}", res);
            assert_ne!(res, "");
        }
    }
}
