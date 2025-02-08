use async_trait::async_trait;

use crate::utils::ToolCallingError;

use super::{Tool, ToolBuilder};

pub struct HumanIntervene {
    content: String,
    response: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CallArgs {
    help: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Response {
    response: String,
}

#[async_trait]
impl Tool for HumanIntervene {
    fn name(&self) -> &str {
        "HumanIntervene"
    }

    fn tooldoc(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "human_intervene",
                "description": "Call this tool if you need help from human. e.g. dangerous operation, lack of necessary tools, etc. Make sure you have done every effort before contacting the human, do not ask for trivial help.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "help": {
                            "type": "string",
                            "description": "explain to be get from human."
                        },
                    },
                    "required": ["help"],
                    "additionalProperties": false
                }
            }
        })
    }

    async fn call(&mut self, arg_string: String) -> Result<String, crate::utils::ToolCallingError> {
        let call_args: CallArgs = serde_json::from_str(&arg_string).map_err(|e| {
            crate::utils::ToolCallingError::new(format!(
                "Calling {} error: {}",
                self.name(),
                e.to_string()
            ))
        })?;

        self.content = call_args.help;

        // TODO: request external tools.
        // TODO: How to wakeup certain async future from outside interruption?

        let resp = Response {
            response: self.response.clone(),
        };
        serde_json::to_string(&resp)
            .map_err(|e| ToolCallingError::new(format!("Error marshalling response: {}", e)))
    }

    fn fork(&self, args: Vec<String>) -> Result<Box<dyn Tool>, crate::utils::ToolForkingError> {
        Ok(Box::new(HumanIntervene::default()))
    }
}

impl Into<HumanIntervene> for ToolBuilder {
    fn into(self) -> HumanIntervene {
        HumanIntervene::default()
    }
}

impl Default for HumanIntervene {
    fn default() -> Self {
        HumanIntervene {
            content: "".to_string(),
            response: "".to_string(),
        }
    }
}
