use std::env;

use log::debug;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage},
        tools::{ToolFunctionInfo, ToolInfo, ToolType},
    },
    Ollama,
};
use schemars::schema::RootSchema;

use crate::application::tools::execute_tool;

use super::{ModeratorFeedback, NO_ACTION};

pub const ASSISTANT_PROMPT_FILE: &str = "./assistant_role_definition.md";

#[derive(Clone, Default)]
pub struct Assistant {
    model_name: String,
    ollama: Ollama,
    tool_infos: Vec<ToolInfo>,
    tool_prompt_template: String,
}

fn assemble_tool_prompt_template(tool_prompt_template: &str) -> String {
    let output_message_json = serde_json::to_string(&ModeratorFeedback {
        moderator: String::from("<Name of the moderator>"),
        message: String::from("<Message of the moderator where the instructions should be extracted>"),
        user_id: String::from("<User id which moderator talking to>"),
        chat_id: String::from("<Chat id of the current chat>"),
    }).unwrap();
    let mut template = tool_prompt_template.trim().replace("{NO_ACTION}", NO_ACTION);
    template.push_str("\n\n");
    template.push_str("Input message:");
    template.push_str("\n\n");
    template.push_str(&output_message_json);
    template
}

impl Assistant {
    pub fn new(tool_prompt_template: &str) -> Self {
        let ollama_client = Ollama::new(
            env::var("OLLAMA_HOST_ADDR").unwrap_or(String::from("http://localhost")),
            env::var("OLLAMA_PORT")
                .unwrap_or(String::from("11434"))
                .parse()
                .unwrap(),
        );
        let model_name = env::var("LLM_MODEL").unwrap_or(String::from("mistral-nemo:12b"));

        Self {
            model_name,
            ollama: ollama_client,
            tool_infos: Vec::default(),
            tool_prompt_template: assemble_tool_prompt_template(tool_prompt_template),
        }
    }

    pub fn add_tool(&mut self, name: String, description: String, parameters: RootSchema) {
        let tool_info = ToolInfo {
            tool_type: ToolType::Function,
            function: ToolFunctionInfo {
                name,
                description,
                parameters,
            },
        };
        self.tool_infos.push(tool_info);
    }

    pub async fn validate_chat(
        &mut self,
        input: &str,
    ) -> std::result::Result<String, anyhow::Error> {
        let mut history: Vec<ChatMessage> = Vec::new();

        history.push(ChatMessage::system(self.tool_prompt_template.clone()));

        let response = self
            .ollama
            .send_chat_messages_with_history(
                &mut history,
                ChatMessageRequest::new(
                    self.model_name.to_owned(),
                    vec![ChatMessage::user(input.to_string())],
                )
                .tools(self.tool_infos.clone()),
            )
            .await?;
        debug!("History: {:#?}", history);

        if !response.message.tool_calls.is_empty() {
            for call in response.message.tool_calls {
                let args = call.function.arguments;
                let name: String = call.function.name;
                let rs = execute_tool(name.as_str(), args).await;
                if let Ok(tool_rs) = rs {
                    history.push(ChatMessage::tool(tool_rs));
                }
            }
            let final_response = self
                .ollama
                .send_chat_messages(ChatMessageRequest::new(
                    self.model_name.to_owned(),
                    history.clone(),
                ))
                .await?;
            debug!("Response from tool - History: {:#?}", history);
            return Ok(final_response.message.content);
        }
        Ok(response.message.content)
    }
}

#[cfg(test)]
mod assistant_test {
    use log::debug;
    use mobot::init_logger;
    use schemars::schema_for;

    use crate::application::assistant_agent::{Assistant, ASSISTANT_PROMPT_FILE};

    use crate::application::tools::{
        MUTE_MEMBER, MUTE_MEMBER_DESCRIPTION, WEB_SEARCH, WEB_SEARCH_DESCRIPTION,
    };
    use crate::application::{tools, ModeratorFeedback};

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
    async fn should_test_tool_feature_successfully() {
        init_logger();
        let mut assistant = Assistant::new(&read_prompt_template(ASSISTANT_PROMPT_FILE));
        assistant.add_tool(
            WEB_SEARCH.to_string(),
            WEB_SEARCH_DESCRIPTION.to_string(),
            schema_for!(tools::WebSearchParams),
        );
        assistant.add_tool(
            MUTE_MEMBER.to_string(),
            MUTE_MEMBER_DESCRIPTION.to_string(),
            schema_for!(tools::MuteMemberParams),
        );
        let request = ModeratorFeedback {
            user_id: "1".to_string(),
            chat_id: "1".to_string(),
            moderator: "Kate".to_string(),
            message: "Ok, ich suche nach der aktuellen Neuigkeiten in der Weltwirtschaft"
                .to_string(),
        };
        let input_json = serde_json::to_string(&request).unwrap();
        let response = assistant
            .validate_chat(input_json.as_str())
            .await;

        let Ok(res) = response else {
            panic!("Failed to get response2");
        };
        debug!("Response: {}", res);
    }
}
