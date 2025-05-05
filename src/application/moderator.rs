use log::debug;
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use std::{collections::VecDeque, env, error::Error, vec};

const MAX_HISTORY_BUFFER_SIZE: usize = 50;
pub const NO_ACTION: &str = "[NO ACTION]";

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
        let model_name = env::var("LLM_MODEL").unwrap_or(String::from("mistral-nemo:12b"));

        let messages = vec![ChatMessage::system(format!(
            "As an AI assistant in a german speaking Telegram group, your name is {} and your role is supporting the admins as a moderator in different channels to prevent members using vulgar expression, fall into hot discussions or blaming each other. The preferred language in the chat group is German.
Your tasks looks as follows:
1. keep the discussions in dedicated channels peacefully.
2. If the members not following the rules repeatedly, give them an advise.
Your need to response to the members if the following conditions apply:
1. if you see the members using vulgar expression
2. if they asking you directly
3. if they insulting each other
If none of the condition applied just answer exactly with: {}",
            name,
            NO_ACTION
        ))];
        let history_buffer = HistoryBuffer::new(messages);

        Self {
            model_name,
            ollama: ollama_client,
            history_buffer,
            administrators: Vec::new(),
        }
    }

    pub async fn chat_forum(
        &mut self,
        topic: String,
        username: String,
        message: String,
    ) -> Result<String, Box<dyn Error>> {
        let user_message = ChatMessage::user(format!(
            "Channel: {}, User: {}, Message: {}",
            topic, username, message
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
        debug!("History after chat: {:#?}", history);
        self.history_buffer.set_message_adjust_buffer(history);
        Ok(response.message.content)
    }

    pub async fn summerize_chat(&self, topic: String) -> Result<String, Box<dyn Error>> {
        let user_message = ChatMessage::user(format!(
            "Summarize what happened in the channel {} in the past in german language please.",
            topic
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

    pub async fn introduce_moderator(&self) -> Result<String, Box<dyn Error>> {
        let mut history = self.history_buffer.get_initial_prompt_messages();
        history.push(ChatMessage::user("Introduce yourself in the group in german".to_string()));

        debug!("History: {:#?}", history);

        let response = self
            .ollama
            .send_chat_messages(ChatMessageRequest::new(self.model_name.to_owned(), history))
            .await?;
        Ok(response.message.content)
    }

    pub fn register_administrator(&mut self, username: String) {
        self.administrators.push(username);
    }

    pub fn is_administrator(&self, username: &str) -> bool {
        self.administrators.contains(&username.to_string())
    }
}

#[cfg(test)]
mod moderator_test {

    use mobot::init_logger;

    use crate::application;

    use super::*;

    #[tokio::test]
    async fn should_test_moderator_successfully() {
        let mut moderator = Moderator::new("Kate");
        init_logger();
        let rs1 = moderator
            .chat_forum(
                "Blume".to_string(),
                "Sabine".to_string(),
                "Hallo Leute, gehts euch gut?".to_string(),
            )
            .await;

        let rs2 = moderator
            .chat_forum(
                "Blume".to_string(),
                "Steffen".to_string(),
                "Sabine ist dumm :)".to_string(),
            )
            .await;

        let rs3 = moderator
            .chat_forum(
                "Blume".to_string(),
                "Sabine".to_string(),
                "Steffen du bist selber dumm!".to_string(),
            )
            .await;

        let rs4 = moderator
            .chat_forum(
                "Blume".to_string(),
                "Kevin".to_string(),
                "Hallo Kate in welchen Channel sind wir gerade?".to_string(),
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
        let mut moderator = Moderator::new("Kate");
        init_logger();
        let forum_name = "Spiel und Spaß".to_string();
        let _ = moderator
            .chat_forum(
                forum_name.clone(),
                "Sabine".to_string(),
                "Hallo Leute, gehts euch gut?".to_string(),
            )
            .await;
        let _ = moderator
            .chat_forum(
                forum_name.clone(),
                "Kevin".to_string(),
                "Jau alles bestens".to_string(),
            )
            .await;
        let _ = moderator
            .chat_forum(
                forum_name.clone(),
                "Steffi".to_string(),
                "Wo ist Steffen in letzter Zeit?".to_string(),
            )
            .await;
        let _ = moderator
            .chat_forum(
                forum_name.clone(),
                "Sabine".to_string(),
                "Keine Ahnung wo er steck".to_string(),
            )
            .await;
        let _ = moderator
            .chat_forum(
                forum_name.clone(),
                "Kevin".to_string(),
                "Der hat Urlaub gerade auf der Karibik hehe :)".to_string(),
            )
            .await;
        let _ = moderator
            .chat_forum(
                forum_name.clone(),
                "Sabine".to_string(),
                "Schön da möchte ich auch mal hin".to_string(),
            )
            .await;
        let _ = moderator
            .chat_forum(
                "Azure".to_string(),
                "Conrad".to_string(),
                "Was passiert gerade in der Cloud?".to_string(),
            )
            .await;
        let _ = moderator
            .chat_forum(
                "Azure".to_string(),
                "Morice".to_string(),
                "Keine Ahnung, wahrscheinlich gab es dort einen update".to_string(),
            )
            .await;

        let rs = moderator.summerize_chat(forum_name).await;
        if let Ok(res) = rs {
            debug!("{}", res);
        }
    }
}
