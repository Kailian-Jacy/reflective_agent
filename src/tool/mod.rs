pub mod draft;
pub mod human;
pub mod result;
pub mod shell;

use crate::utils::{ToolCallingError, ToolForkingError};
use async_trait::async_trait;
use draft::Draft;
use human::HumanIntervene;
use result::TaskEnds;
use serde::{Deserialize, Serialize};
use shell::Shell;

// Initiate tools.
pub fn available_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(Into::<Draft>::into(ToolBuilder {
            name: "draft".to_string(),
            args: vec![],
        })),
        Box::new(Into::<Shell>::into(ToolBuilder {
            name: "shell".to_string(),
            args: vec![],
        })),
        Box::new(Into::<HumanIntervene>::into(ToolBuilder {
            name: "humanIntervene".to_string(),
            args: vec![],
        })),
        Box::new(Into::<TaskEnds>::into(ToolBuilder {
            name: "taskEnds".to_string(),
            args: vec![],
        })),
    ]
}

#[async_trait]
pub trait Tool {
    fn name(&self) -> &str;
    fn tooldoc(&self) -> serde_json::Value;
    async fn call(&mut self, arg_string: String) -> Result<String, ToolCallingError>;
    // Fork tool from runtime version to task version.
    fn fork(&self, args: Vec<String>) -> Result<Box<dyn Tool>, ToolForkingError>;
}

#[derive(Clone, Deserialize)]
pub struct ToolBuilder {
    pub name: String,
    pub args: Vec<String>, // Args to be invoked in the task configuration.
}

#[derive(Serialize)]
struct Function {
    #[serde(rename = "type")]
    func_type: String,
    function: FunctionDetails,
}

#[derive(Serialize)]
struct FunctionDetails {
    name: String,
    description: String,
    parameters: Parameters,
}

#[derive(Serialize)]
struct Parameters {
    #[serde(rename = "type")]
    param_type: String,
    properties: Properties,
    required: Vec<String>,
    #[serde(rename = "additionalProperties")]
    additional_properties: bool,
}

#[derive(Serialize)]
struct Properties {
    executable: Executable,
    args: Args,
}

#[derive(Serialize)]
struct Executable {
    #[serde(rename = "type")]
    exec_type: String,
    description: String,
}

#[derive(Serialize)]
struct Args {
    #[serde(rename = "type")]
    args_type: String,
    description: String,
    #[serde(rename = "minItems")]
    min_items: u32,
    #[serde(rename = "maxItems")]
    max_items: u32,
    items: Items,
}

#[derive(Serialize)]
struct Items {
    #[serde(rename = "type")]
    item_type: String,
}
