use async_trait::async_trait;

use crate::{provider::ToolCallFunction, utils::ToolCallingError};

use super::{Tool, ToolBuilder};

#[derive(Clone)]
pub struct Draft {
    base: ToolBuilder,
    buffer: String,
    plan: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CallArgs {
    content: String,
    plan: Option<PlanArgs>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PlanArgs {
    if_update: bool,
    full_content: Option<String>,
}

#[async_trait]
impl Tool for Draft {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn tooldoc(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "record_content",
                "description": "You'll forget anything after your answer except content recorded to this script paper. So keep ANYTHING important to your task goal, including plan and progress, findings about the environment, and other things. Call this tool to record. Repeat original content on this paper and skip outdated or useless content to delete. Those content originally on paper will be lost if you don't repeat. Summarize useful information you get this round. Revise the plan you made last round to fit the progress.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Updated content to be recorded on the script paper."
                        },
                        "plan": {
                            "type": "object",
                            "properties": {
                                "if_update": {
                                    "type": "bool",
                                    "description": "If you want to edit the current plan"
                                },
                                "full_content": {
                                    "type": "string",
                                    "description": "Only not empty if `if_update` is true. Give your full updated plan as a string."
                                }
                            }
                        }
                    },
                    "required": ["content"],
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

        self.buffer = call_args.content;

        if let Some(p) = call_args.plan {
            if p.if_update {
                if let Some(content) = p.full_content {
                    self.plan = content;
                } else {
                    return Err(ToolCallingError::new("illegal tool call: plan.full_content should not be none if plan.if_update is true.".to_string()));
                }
            }
        }

        // No need to give feed back in case successful. The content will be put to prompt.
        Ok("".to_string())
    }

    fn fork(&self, args: Vec<String>) -> Result<Box<dyn Tool>, crate::utils::ToolForkingError> {
        Ok(Box::new(self.clone()))
    }
}

impl Into<Draft> for ToolBuilder {
    fn into(self) -> Draft {
        Draft {
            base: self,
            buffer: "".to_string(),
            plan: "1. Review task target and make possible planning.".to_string(),
        }
    }
}
