use super::{MessageInput, ModeratorMessage};
use log::debug;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        parameters::FormatType,
    },
    Ollama,
};
use std::env;
use std::{collections::VecDeque, vec};

const MAX_HISTORY_BUFFER_SIZE: usize = 60;
pub const NO_ACTION: &str = "NO_ACTION";
pub const MODERATOR_PROMPT_FILE: &str = "./moderator_role_definition.md";

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
}

fn assemble_moderator_prompt_template(name: &str, prompt_template: &str) -> String {
    let input_message_json = serde_json::to_string(&MessageInput {
        channel: String::from("<Name of the channel>"),
        user_role: String::from("<Role of the user in the chat>"),
        user_id: String::from("<User identity as numbers>"),
        chat_id: String::from("<Chat identity as numbers>"),
        user: String::from("<Name of the member>"),
        message: String::from("<Text message>"),
    })
    .unwrap();
    let output_message_json = serde_json::to_string(&ModeratorMessage {
        moderator: String::from("<Name of the moderator>"),
        message: String::from("<Moderator message>"),
        user_id: String::from("<User identity of the user who sent the message to the moderator>"),
        chat_id: String::from("<Chat identity where moderator and user chat together>"),
    })
    .unwrap();

    let no_action_message = serde_json::to_string(&ModeratorMessage {
        moderator: name.to_string(),
        message: format!("[{NO_ACTION}]"),
        user_id: String::from("<User identity of the user who sent the message to the moderator>"),
        chat_id: String::from("<Chat identity where the moderator and the users are in>"),
    })
    .unwrap();

    let mut moderator_template: String = prompt_template
        .trim()
        .replace("{name}", name)
        .replace("{NO_ACTION}", no_action_message.as_str());
    moderator_template.push_str("\n\n## Format \n\n");
    moderator_template.push_str("Input format as valid JSON: \n\n");
    moderator_template.push_str(&input_message_json);
    moderator_template.push_str("\n\n");
    moderator_template.push_str("Output format as valid JSON: \n\n");
    moderator_template.push_str(&output_message_json);
    moderator_template.push_str("\n\n");
    moderator_template
}

impl Moderator {
    pub fn new(name: &str, moderator_prompt_template: &str) -> Self {
        let ollama_client = Ollama::new(
            env::var("OLLAMA_HOST_ADDR").unwrap_or(String::from("http://localhost")),
            env::var("OLLAMA_PORT")
                .unwrap_or(String::from("11434"))
                .parse()
                .unwrap(),
        );
        let model_name = env::var("LLM_MODEL").unwrap_or(String::from("mistral-nemo:12b"));

        let messages = vec![ChatMessage::system(assemble_moderator_prompt_template(
            name,
            moderator_prompt_template,
        ))];
        let history_buffer = HistoryBuffer::new(messages);

        Self {
            model_name,
            ollama: ollama_client,
            history_buffer,
        }
    }

    pub async fn chat_forum(
        &mut self,
        input_json: &str,
    ) -> std::result::Result<String, anyhow::Error> {
        let user_message = ChatMessage::user(input_json.to_string());
        let mut history = self.history_buffer.get_history();
        debug!("History before chat: {:#?}", history);
        let response = self
            .ollama
            .send_chat_messages_with_history(
                &mut history,
                ChatMessageRequest::new(self.model_name.to_owned(), vec![user_message])
                    .format(FormatType::Json),
            )
            .await?;

        debug!("History: {:#?}", history);
        self.history_buffer.set_message_adjust_buffer(history);
        Ok(response.message.content)
    }

    pub async fn summarize_chat(&self, topic: &str) -> std::result::Result<String, anyhow::Error> {
        let user_message = ChatMessage::user(format!(
            "Only summarize the conversations from the channel: {} in german language please. Please don't mention the channel name in the summary.",
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

    pub async fn introduce_moderator(&self) -> std::result::Result<String, anyhow::Error> {
        let mut history = self.history_buffer.get_initial_prompt_messages();
        history.push(ChatMessage::user(
            "Introduce yourself and tell the members what are the rules in this group in german"
                .to_string(),
        ));

        debug!("History: {:#?}", history);

        let response = self
            .ollama
            .send_chat_messages(
                ChatMessageRequest::new(self.model_name.to_owned(), history)
                    .format(FormatType::Json),
            )
            .await?;
        Ok(response.message.content)
    }
}

#[cfg(test)]
mod moderator_test {

    use mobot::init_logger;

    use crate::application::moderator_agent::{Moderator, MODERATOR_PROMPT_FILE};

    use super::*;

    fn read_prompt_template(path: &str) -> String {
        let template = std::fs::read_to_string(path);
        match template {
            Ok(content) => content,
            Err(e) => {
                panic!("Failed to read the prompt template file: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn should_test_moderator_successfully() {
        let mut moderator = Moderator::new("Kate", &read_prompt_template(MODERATOR_PROMPT_FILE));
        init_logger();
        let rs1 = moderator
            .chat_forum(r#"{ "channel": "Play & Fun", "user_role": "Regular User", "user_id:" "1", "chat_id": "56789",  "user": "Sabine", "message": "Hallo Leute, gehts euch gut?" }"#)
            .await;
        let rs2 = moderator
            .chat_forum(
                r#"{ "channel": "Play & Fun", "user_role": "Regular User", "user_id:" "2", "chat_id": "56789", "user": "Steffen", "message": "Sabine ist dumm :)" }"#)
            .await;
        let rs3 = moderator
            .chat_forum(r#"{ "channel": "Play & Fun", "user_role": "Regular User",  "user_id:" "1", "chat_id": "56789", "user": "Sabine", "message": "Steffen du bist selber dumm!" }"#)
            .await;

        let rs4 = moderator
            .chat_forum(r#"{ "channel": "Play & Fun", "user_role": "Regular User", "user_id:" "3", "chat_id": "56789",  "user": "Kevin", "message": "Hallo Kate in welchen Channel sind wir gerade?" }"#)
            .await;

        let rs5 = moderator
            .chat_forum(r#"{ "channel": "Play & Fun", "user_role": "Regular User", "user_id:" "3", "chat_id": "56789", "user": "Kevin", "message": "ich frage mich wo Fuffi ist?" }"#)
            .await;

        if let Ok(res) = rs1 {
            debug!("{}", res);
            assert_ne!(res, NO_ACTION);
        }
        if let Ok(res) = rs2 {
            debug!("{}", res);
            assert_ne!(res, NO_ACTION);
        }
        if let Ok(res) = rs3 {
            debug!("{}", res);
            assert_ne!(res, NO_ACTION);
        }
        if let Ok(res) = rs4 {
            debug!("{}", res);
            assert_ne!(res, NO_ACTION);
        }
        if let Ok(res) = rs5 {
            debug!("{}", res);
            assert_ne!(res, NO_ACTION);
        }
    }

    #[tokio::test]
    async fn should_test_admin_support_successfully() {
        let mut moderator = Moderator::new("Kate", &read_prompt_template(MODERATOR_PROMPT_FILE));
        init_logger();
        let channel_id = "Play & Fun";
        let mut message1 = serde_json::to_string(&MessageInput {
                channel: channel_id.to_string(),
                user_role: "Regular User".to_string(),
                user: "Kevin".to_string(),
                user_id: "123".to_string(),
                chat_id: "56789".to_string(),
                message: "Was will Steffen von uns?".to_string(),
            }).unwrap();
        let _ = moderator.chat_forum(message1.as_str()).await;
        message1 = serde_json::to_string(&MessageInput {
                channel: channel_id.to_string(),
                user_role: "Admin".to_string(),
                user: "LL".to_string(),
                user_id: "1".to_string(),
                chat_id: "56339".to_string(),
                message: "Hey Kevin, lasst es doch sein darüber zu lästern".to_string(),
            }).unwrap();
        let rs = moderator.chat_forum(message1.as_str()).await;
        if let Ok(res) = rs {
            debug!("{}", res);
        }
    }

    #[tokio::test]
    async fn should_test_moderator_summerize_chat_successfully() {
        let mut moderator = Moderator::new("Kate", &read_prompt_template(MODERATOR_PROMPT_FILE));

        init_logger();

        let channel_id = "Have Fun";
        let _ = moderator
            .chat_forum(r#"{ "channel": "Have Fun", "user_role": "Regular User", "user_id:" "1", "chat_id": "12345", "user": "Sabine", "message": "Hallo Leute, gehts euch gut?" }"#)
            .await;
        let _ = moderator
            .chat_forum(
                r#"{ "channel": "Have Fun", "user_role": "Regular User", "user_id:" "2", "chat_id": "12345", "user": "Kevin", "message": "Jau alles bestens" }"#,
            )
            .await;
        let _ = moderator
            .chat_forum(r#"{ "channel": "Have Fun", "user_role": "Regular User", "user_id:" "3", "chat_id": "12345", "user": "Steffi", "message": "Wo ist Steffen in letzter Zeit?" }"#)
            .await;
        let _ = moderator
            .chat_forum(r#"{ "channel": "Have Fun", "user_role": "Regular User", "user_id:" "1", "chat_id": "12345", "user": "Sabine", "message": "Keine Ahnung wo er steck" }"#)
            .await;
        let _ = moderator
            .chat_forum(r#"{ "channel": "Have Fun", "user_role": "Regular User", "user_id:" "2", "chat_id": "12345",  "user": "Kevin", "message": "Der hat Urlaub gerade auf der Karibik hehe :)" }"#)
            .await;
        let _ = moderator
            .chat_forum(r#"{ "channel": "Have Fun", "user_role": "Regular User", "user_id:" "1", "chat_id": "12345",  "user": "Sabine", "message": "Schön da möchte ich auch mal hin" }"#)
            .await;
        let _ = moderator
            .chat_forum(r#"{ "channel": "Cloud Stuffs", "user_role": "Regular User", "user_id:" "4", "chat_id": "4321",  "user": "Conrad", "message": "Was passiert gerade in der Cloud?" }"#)
            .await;
        let _ = moderator
            .chat_forum(r#"{ "channel": "Cloud Stuffs", "user_role": "Regular User", "user_id:" "5", "chat_id": "4321", "user": "Morice", "message": "Keine Ahnung, wahrscheinlich gab es dort einen update" }"#)
            .await;

        let rs = moderator.summarize_chat(channel_id).await;
        if let Ok(res) = rs {
            debug!("{}", res);
            assert!(!res.contains("Cloud") && !res.contains("update"));
        }
    }
}
