// this is a shell isolated in container for llm to execute related works.
// it should be exposed as the form of tool. but now, let's make it simple.

use async_trait::async_trait;

use crate::tool::Tool;
use crate::utils::{ShellRunningError, ToolCallingError};
use std::process::{Command, Output};

use super::ToolBuilder;

// Main runner.
#[derive(Clone)]
pub struct Shell {
    base: ToolBuilder,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CallArgs {
    executable: String,
    args: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Response {
    stdout: String,
    stderr: String,
    status_code: i32,
}

#[async_trait]
impl Tool for Shell {
    fn name(&self) -> &str {
        &self.base.name
    }

    // return pre-coded tooldoc.
    fn tooldoc(&self) -> serde_json::Value {
        serde_json::json!({
          "type": "function",
          "function": {
            "name": "search_products",
            "description": "This is a shell can be called to run code related to your works. If you find anything related to the environment unknown or uninstalled, take several turns to detect or install it first.",
            "parameters": {
              "type": "object",
              "properties": {
                "executable": {
                  "type": "string",
                  "description": "executable to be called. Do not put arguments here.",
                },
                "args": {
                  "type": "array",
                  "description": "argument list to that array",
                  "minItems": 1,
                  "maxItems": 1000,
                  "items": {
                    "type": "string"
                  }
                }
              },
              "required": ["executable"],
              "additionalProperties": false
            }
          }
        })
    }

    async fn call(&mut self, arg_string: String) -> Result<String, ToolCallingError> {
        let call_args: CallArgs = serde_json::from_str(&arg_string).map_err(|e| {
            crate::utils::ToolCallingError::new(format!(
                "Calling {} error: {}",
                self.name(),
                e.to_string()
            ))
        })?;

        match self.run(call_args.executable, call_args.args) {
            Ok((stdout, _stderr, _status_code)) => {
                let resp = Response {
                    stdout,
                    stderr: _stderr,
                    status_code: _status_code,
                };
                serde_json::to_string(&resp).map_err(|e| {
                    ToolCallingError::new(format!("Error marshalling response: {}", e))
                })
            }
            Err(e) => Err(ToolCallingError::new(format!("Error: {}", e))),
        }
    }

    fn fork(&self, args: Vec<String>) -> Result<Box<dyn Tool>, crate::utils::ToolForkingError> {
        // TODO: Start a new sandbox and yield the shell.
        Ok(Box::new(self.clone()))
    }
}

impl Into<Shell> for ToolBuilder {
    fn into(self) -> Shell {
        Shell { base: self }
    }
}

impl Shell {
    pub fn run(
        &mut self,
        exe: String,
        args: Vec<String>,
    ) -> Result<(String, String, i32), ShellRunningError> {
        if args.len() == 0 {
            return Err(ShellRunningError::new("zero shell command.".to_string()));
        }

        let output: Output = Command::new(exe)
            .args(&args)
            .output()
            .map_err(|e| ShellRunningError::new(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        // will be none if signal disrupted. not possible for now.
        let status_code = output.status.code().unwrap();

        Ok((stdout, stderr, status_code))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_run_echo_command() {
        let mut shell: Shell = ToolBuilder {
            name: "shell".to_string(),
            args: vec![],
        }
        .into();
        let call_args = CallArgs {
            executable: "echo".to_string(),
            args: vec!["Hello, World!".to_string()],
        };
        let command = serde_json::to_string(&call_args).expect("Failed to serialize CallArgs");
        let result: Response = serde_json::from_str(&shell.call(command).await.unwrap()).unwrap();

        assert_eq!(result.stdout.trim(), "Hello, World!");
        assert_eq!(result.stderr.trim(), "");
        assert_eq!(result.status_code, 0);
    }
}
