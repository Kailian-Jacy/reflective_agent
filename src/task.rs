use std::{fs::File, io::BufReader, path::Path};

use crate::{
    model::Model,
    tool::{Tool, ToolBuilder},
};

#[derive(serde::Deserialize)]
pub struct Task {
    pub name: String,
    #[serde(rename = "model")]
    pub model_name: String,
    pub target: String,
    pub tools: Vec<ToolBuilder>,
    pub max_iterations: usize,
}

impl Task {
    // Read from json file and parse into it.
    pub fn from_path<P>(path: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let task: Task = serde_json::from_reader(reader)?;
        Ok(task)
    }
}
