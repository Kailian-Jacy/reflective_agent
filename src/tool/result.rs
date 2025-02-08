use async_trait::async_trait;

use super::{Tool, ToolBuilder};

pub struct TaskEnds {
    status: bool,
    result: String,
}

#[derive(serde::Deserialize)]
struct CallArgs {
    is_success: bool,
    explanation: String,
}

#[async_trait]
impl Tool for TaskEnds {
    fn name(&self) -> &str {
        "TaskEnds"
    }

    fn tooldoc(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "task_ends",
                "description": "Call this tool when task progress ends, and give result. Remind, you should have done every effort to progress the task before calling this tool, and the task ends.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "is_success": {
                            "type": "bool",
                            "description": "True for success and false for failure."
                        },
                        "explanation": {
                            "type": "string",
                            "description": "Result for success and reason for failure. On success the result should be brief and compact outcome part and followed by the report of the work digest you have done so far. On failure, explain failure reason and provide possible promotion suggestion;",
                        }
                    },
                    "required": ["is_success", "explanation"],
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

        self.status = call_args.is_success;
        self.result = call_args.explanation;

        Ok(format!("Task ended with status: {}", self.status))
    }

    fn fork(&self, args: Vec<String>) -> Result<Box<dyn Tool>, crate::utils::ToolForkingError> {
        Ok(Box::new(TaskEnds::default()))
    }
}

impl Into<TaskEnds> for ToolBuilder {
    fn into(self) -> TaskEnds {
        TaskEnds::default()
    }
}

impl Default for TaskEnds {
    fn default() -> Self {
        TaskEnds {
            status: false,
            result: "".to_string(),
        }
    }
}
