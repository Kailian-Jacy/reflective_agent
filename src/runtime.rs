use futures::prelude::*;
use std::cell::RefCell;
use std::sync::Mutex;

use crate::{
    config::Config,
    model::Model,
    provider::{Message, Request, Response, Roles},
    task::Task,
    tool::{available_tools, Tool},
    utils::{ModelNotRegistered, ToolCallingError, ToolNotRegistered},
};

pub struct Runtime {
    config: Config,
    models: Vec<Mutex<Model>>,
    tools: Vec<Box<dyn Tool>>,
    tasks: Vec<RuntimeTask>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let models = config
            .to_models()?
            .into_iter()
            .map(|m| Mutex::new(m))
            .collect();
        let tools = available_tools();
        Ok(Runtime {
            config,
            models,
            tools,
            tasks: Vec::new(),
        })
    }

    // TODO: Adding to queue and scheduling task.
    pub fn new_task(&mut self, task: Task) -> Result<(), Box<dyn std::error::Error>> {
        let r = RuntimeTask::from_task(self, task)?;
        self.tasks.push(r);
        Ok(())
    }

    // Spawn a new task. Task should be run and paused automatically.
    async fn execute(&mut self, task: Task) -> Result<RuntimeTask, Box<dyn std::error::Error>> {
        // Prepare Task config to runtime internal.
        let runtime_task = RuntimeTask::from_task(self, task)?;
        // Get response from LLM.
        let mut response = {
            // These resources should die early..
            let message = &Message {
                role: Roles::User,
                content: runtime_task.task.target.clone(),
                tool_calls: None,
            };
            let model = unsafe { &(*runtime_task.model) };
            let tools = &runtime_task.tools.iter().map(|t| t.borrow()).collect();
            let request = &Request::new(model.lock().unwrap().name().to_string())
                .add_message(message)
                .add_tools(tools);
            Response::from_u8(&model.lock().unwrap().do_request(request).await?)?
        };
        // Ensemble tool calls before async execution.
        let mut tool_call_pairs = Vec::new();
        for tool_call in response.tool_calls() {
            let tool = runtime_task
                .tools
                .iter()
                .find(|t| t.borrow().name() == tool_call.name)
                .ok_or_else(|| {
                    // TODO: It should not be returned but feedback to LLM.
                    ToolCallingError::new(format!(
                        "required tool not found in runtime: {}",
                        tool_call.name
                    ))
                })?
                .borrow_mut();
            tool_call_pairs.push((tool, tool_call));
        }
        // Async execution of tool calls.
        futures::stream::iter(tool_call_pairs)
            .for_each(|(mut tool, tool_call)| async move {
                tool.call(tool_call.arguments).await;
            })
            .await;
        // TODO: Parse result. Conduct Workflow.
        // TODO: The message resulting tools. Ending tool, human_intervene tool.
        Ok(runtime_task)
    }
}

struct RuntimeTask {
    task: Task,
    history: Vec<RuntimeHistory>,
    model: *const Mutex<Model>,
    tools: Vec<RefCell<Box<dyn Tool>>>,
    status: RuntimeTaskStatus,
    iterations: usize,
}

enum RuntimeTaskStatus {
    NotStarted,
    Running,
    Waiting,
    Ended(bool),
}

impl RuntimeTask {
    pub fn from_task(runtime: &Runtime, task: Task) -> Result<Self, Box<dyn std::error::Error>> {
        let model: *const Mutex<Model> = &*runtime
            .models
            .iter()
            .find(|model| model.lock().unwrap().name() == task.model_name)
            .ok_or_else(|| {
                ModelNotRegistered::new(format!("requested model {} not found", task.model_name))
            })?;
        let mut tools = Vec::new();
        for tool_builder in task.tools.iter() {
            tools.push(
                runtime
                    .tools
                    .iter()
                    .find(|tool| tool.name() == tool_builder.name)
                    .ok_or_else(|| {
                        ToolNotRegistered::new(format!(
                            "requested tool {} not found",
                            tool_builder.name
                        ))
                    })?
                    .fork(tool_builder.args.clone())?,
            )
        }
        Ok(RuntimeTask {
            task,
            history: Vec::new(),
            tools: tools.into_iter().map(|tool| RefCell::new(tool)).collect(),
            model,
            status: RuntimeTaskStatus::NotStarted,
            iterations: 0,
        })
    }
}

struct RuntimeHistory {
    time: std::time::SystemTime,
    request: String,
    response: String,
}
