use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};
use std::{env, error::Error, vec};

const PROMP_TEMPLATE: &str = r#"
[ROLE] Your name is Kate and you are the moderator of a german speaking chat group.
[CONTEXT] As a moderator your goal is preventing users using vulgar expression, hot discussions or insult each other.
[TASK] Observe the discussions in the group and keep it peacefully, otherwise you have to mute and warn the user if he/she is not following the rules three times
"#;

#[derive(Clone, Default)]
pub struct Moderator {
    pub model_name: String,
    ollama: Ollama,
    history: Vec<ChatMessage>,
    administrators: Vec<String>,
}

impl Moderator {
    pub fn new() -> Self {
        let ollama_client = Ollama::new(
            env::var("OLLAMA_HOST_ADDR").unwrap_or(String::from("http://localhost")),
            env::var("OLLAMA_PORT")
                .unwrap_or(String::from("11434"))
                .parse()
                .unwrap(),
        );
        let model_name = env::var("LLM_MODEL").unwrap_or(String::from("llama3.2:latest"));
        let history = vec![
            ChatMessage::system(PROMP_TEMPLATE.to_string())
        ];

        Self {
            model_name,
            ollama: ollama_client,
            history,
            administrators: Vec::new(),
        }
    }

    pub async fn chat(
        &mut self,
        username: String,
        message: String,
    ) -> Result<String, Box<dyn Error>> {
        let response = self
            .ollama
            .send_chat_messages_with_history(
                &mut self.history,
                ChatMessageRequest::new(
                    self.model_name.to_owned(),
                    vec![ChatMessage::user(format!("User: {} Message: {}", username, message))],
                ),
            )
            .await?;
        Ok(format!(
            "Role: {:?} Message: {}",
            response.message.role,
            response.message.content
        ))
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

    use super::*;

    #[tokio::test]
    async fn should_test_moderator_successfully() {
        let mut moderator = Moderator::new();
        let rs1 = moderator
            .chat("Fuffi".to_string(), "Hallo Leute, gehts euch gut?".to_string())
            .await;

        let rs2 = moderator
            .chat("Steffen".to_string(), "Fuffi ist schwul".to_string())
            .await;

        let rs3 = moderator
        .chat("Fuffi".to_string(), "Steffen du bist selber schwul!".to_string())
        .await;

        if let Ok(res) = rs1 {
            println!("{}", res);
            assert_ne!(res, "");
        }
        if let Ok(res) = rs2 {
            println!("{}", res);
            assert_ne!(res, "");
        }
        if let Ok(res) = rs3 {
            println!("{}", res);
            assert_ne!(res, "");
        }
    }
}
