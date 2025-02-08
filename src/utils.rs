use simple_logger::SimpleLogger;
use std::fmt;

// Model picking and Tool binding error.
pub type ModelNotRegistered = Errorbase;
pub type ToolNotRegistered = Errorbase;
pub type ProviderNotRegistered = Errorbase;

// Service Error.
pub type ProviderError = Errorbase;
pub type ProviderResponseUnmarshalError = Errorbase;
pub type ProviderResponseError = Errorbase;

// ToolCalls Error.
pub type ToolCallingError = Errorbase;
pub type ToolForkingError = Errorbase;
pub type ShellRunningError = Errorbase;

#[derive(Debug)]
pub struct Errorbase {
    content: String,
}

impl Errorbase {
    pub fn new(content: String) -> Self {
        Errorbase { content }
    }
}

impl fmt::Display for Errorbase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl std::error::Error for Errorbase {}

pub fn log_init() {
    SimpleLogger::new().init().unwrap();
    log::info!("Initiated logger.")
}
