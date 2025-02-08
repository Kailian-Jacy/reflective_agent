use crate::{model::Model, provider::Provider, utils::ProviderNotRegistered};
use std::path::Path;

#[derive(serde::Deserialize)]
pub struct Config {
    models: Vec<ModelParser>,
    services: Vec<ServiceParser>,
    // TODO: Add notifier to interact with human. Telegram bot, mail, CLI.
    // human_notifier: Vec<NotiferParser>,
}

impl Config {
    pub fn from_file<P>(path: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let config = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&config)?;
        Ok(config)
    }

    pub fn to_models(&self) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
        let services = self
            .services
            .iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect::<std::collections::HashMap<String, ServiceParser>>();
        let mut models = Vec::new();
        for model_parser in &self.models {
            models.push(
                services
                    .get(&model_parser.provider)
                    .ok_or_else(|| {
                        ProviderNotRegistered::new(format!(
                            "Service {} required by {} is not registered.",
                            model_parser.provider, model_parser.name
                        ))
                    })
                    .map(|s| Model::new(&model_parser.name, (*s).clone()))?,
            )
        }
        Ok(models)
    }
}

#[derive(serde::Deserialize)]
struct ModelParser {
    pub name: String,
    pub provider: String,
}

type ServiceParser = Provider;

