use std::ops::Deref;
use std::sync::Mutex;

use serde_json::json;

use crate::model::Model;
use crate::{
    tool::Tool,
    utils::{ProviderError, ProviderResponseError},
};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Provider {
    pub name: String,
    ip: String,
    port: u16,
}

impl Provider {
    pub fn new(name: String, ip: String, port: u16) -> Self {
        Provider { name, ip, port }
    }

    // Feed the request to llm and get response.
    pub async fn do_request<'a>(&self, request: &Request<'a>) -> Result<Vec<u8>, ProviderError> {
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}/v1/chat/completions", self.ip, self.port);
        log::info!("111");
        let body = request.format().await;
        log::info!("Body: {}", body);

        // Actually make the request
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| ProviderError::new(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::new(e.to_string()))?;

        Ok(bytes.to_vec())
    }
}

// Message and Roles in Response and Request.
#[derive(serde::Deserialize)]
pub struct Message {
    pub(crate) role: Roles,
    #[serde(default)]
    pub(crate) content: String,
    // Used for receiving tool calls request. TODO: Make it non-settable for sending request.
    #[serde(skip_serializing)]
    pub(crate) tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Roles {
    User,
    System,
    Assistant,
}

impl From<&str> for Roles {
    fn from(value: &str) -> Self {
        match value {
            "user" | "User" => Roles::User,
            "system" | "System" => Roles::System,
            "assistant" | "Assistant" => Roles::Assistant,
            _ => panic!("Invalid role"),
        }
    }
}

// Request to be made.
pub struct Request<'a> {
    model: String,
    messages: Vec<&'a Message>,
    tools: Vec<&'a Box<dyn Tool>>,
}

impl<'a> Request<'a> {
    // Construct empty struct Request.
    pub fn new(model: String) -> Self {
        Request {
            model,
            messages: Vec::new(),
            tools: Vec::new(),
        }
    }

    pub(crate) async fn format(&self) -> String {
        let tools: Vec<_> = self.tools.iter().map(|tool| tool.tooldoc()).collect();

        let body = json!({
            "model": self.model.clone(),
            "messages": self.messages.iter().map(|msg| {
                json!({
                    "role": format!("{:?}", msg.role).to_lowercase(),
                    "content": msg.content
                })
            }).collect::<Vec<_>>(),
            "tools": tools,
        });
        body.to_string()
    }

    pub fn add_tool(mut self, tool: &'a Box<dyn Tool>) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn add_tools<P>(mut self, tool: &'a Vec<P>) -> Self
    where
        P: Deref<Target = Box<dyn Tool>>,
    {
        tool.iter().for_each(|tool| self.tools.push(&*tool));
        self
    }

    // Add a single message.
    pub fn add_message(mut self, message: &'a Message) -> Self {
        self.messages.push(message);
        self
    }
}

// Response.
#[derive(serde::Deserialize)]
pub struct Response {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
    system_fingerprint: String,
}

#[derive(serde::Deserialize)]
pub struct Choice {
    index: u64,
    finish_reason: String,
    message: Message,
}

#[derive(serde::Deserialize)]
pub struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_calls_type: String,
    function: ToolCallFunction,
}

#[derive(serde::Deserialize, Clone)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(serde::Deserialize)]
pub struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

impl Response {
    pub fn from_u8(bytes: &Vec<u8>) -> Result<Self, ProviderResponseError> {
        serde_json::from_slice(bytes).map_err(|e| {
            ProviderResponseError::new(format!(
                "Unmarshal llm response error: {}.\nPretty print: {}",
                e,
                String::from_utf8_lossy(bytes).into_owned(),
            ))
        })
    }
}

impl Response {
    // Pure content. Thinking part trimmed.
    //  e.g. <think>\nOkay ....beyond that.\n</think>\n\npong
    //  ->
    //  pong.
    pub fn content(&self) -> String {
        // TODO: Could choices to be empty?
        let content = &self.choices[0].message.content;
        if let Some(pos) = content.rfind("</think>") {
            content[(pos + 8)..].trim().to_string()
        } else {
            content.trim().to_string()
        }
    }

    // Give full response.
    pub fn full(&self) -> String {
        // TODO: Could choices to be empty?
        self.choices[0].message.content.clone()
    }

    // Give tool calls
    pub fn tool_calls(&mut self) -> Vec<ToolCallFunction> {
        self.choices.iter().fold(Vec::new(), |mut acc, c| {
            if let Some(msg_tool_calls) = &c.message.tool_calls {
                acc.extend(msg_tool_calls.iter().map(|tc| tc.function.clone()));
            }
            acc
        })
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Mutex;

    use crate::{
        tool::{shell::Shell, ToolBuilder},
        utils::log_init,
    };

    use super::*;

    #[tokio::test]
    async fn test_model_request() -> Result<(), Box<dyn std::error::Error>> {
        log_init();

        let model = Mutex::new(Model::new(
            "deepseek-r1-distill-qwen-14b@q4_k_m",
            Provider::new("LM Studio".to_string(), "192.168.2.228".to_string(), 1234),
        ));
        let message = &Message {
                role: Roles::from("user"), // Role should be an enum.main
                content: String::from("Do not choose any tools. Do not answer anything else. Just response \"pong\" only."),
                tool_calls: None,
            };
        let tool: Box<dyn Tool> = Box::new(Into::<Shell>::into(ToolBuilder {
            name: "shell".to_string(),
            args: vec![],
        }));
        let request = Request::new(model.lock().unwrap().name().to_string())
            .add_message(message)
            .add_tool(&tool);
        let response = Response::from_u8(&(model.lock().unwrap().do_request(&request).await?))?;
        // Trim reasoning part in the response.
        assert_eq!(response.content(), "pong");
        Ok(())
    }

    #[test]
    fn test_deserialize_tool_call() {
        let json_data = r#"
    {
      "id": "chatcmpl-idphs4avvdqc2yxofanzdb",
      "object": "chat.completion",
      "created": 1738763823,
      "model": "deepseek-r1-distill-qwen-14b@q4_k_m",
      "choices": [
        {
          "index": 0,
          "finish_reason": "tool_calls",
          "message": {
            "role": "assistant",
            "content": "",
            "tool_calls": [
              {
                "id": "592365529",
                "type": "function",
                "function": {
                  "name": "search_products",
                  "arguments": "{\"query\":\"Dell products\",\"category\":\"electronics\",\"max_price\":50}"
                }
              }
            ]
          }
        }
      ],
      "usage": {
        "prompt_tokens": 445,
        "completion_tokens": 221,
        "total_tokens": 666
      },
      "system_fingerprint": "deepseek-r1-distill-qwen-14b@q4_k_m"
    }
    "#;

        let response =
            Response::from_u8(&json_data.as_bytes().to_vec()).expect("Failed to deserialize");

        assert_eq!(response.id, "chatcmpl-idphs4avvdqc2yxofanzdb");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].finish_reason, "tool_calls");
        assert_eq!(
            response.choices[0]
                .message
                .tool_calls
                .as_ref()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            response.choices[0].message.tool_calls.as_ref().unwrap()[0].id,
            "592365529"
        );
        assert_eq!(
            response.choices[0].message.tool_calls.as_ref().unwrap()[0].tool_calls_type,
            "function"
        );
        assert_eq!(
            response.choices[0].message.tool_calls.as_ref().unwrap()[0]
                .function
                .name,
            "search_products"
        );
        assert_eq!(
            response.choices[0].message.tool_calls.as_ref().unwrap()[0]
                .function
                .arguments,
            "{\"query\":\"Dell products\",\"category\":\"electronics\",\"max_price\":50}"
        );
    }
}
