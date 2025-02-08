use crate::{
    config::Config,
    provider::{Provider, Request},
    utils::ProviderError,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Model {
    name: String, // More to go.
    provider: Provider,
}

impl Model {
    // Manually construct.
    pub fn new(name: &str, provider: Provider) -> Self {
        Model {
            name: name.to_string(),
            provider,
        }
    }

    // TODO: Try to adopt cache.
    pub fn from_config(config: &Config) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
        config.to_models()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn do_request<'a>(&self, request: &Request<'a>) -> Result<Vec<u8>, ProviderError> {
        self.provider.do_request(request).await
    }
}
