mod config;
mod model;
mod provider;
mod runtime;
mod task;
mod tool;
mod utils;

use config::Config;
use runtime::Runtime;
use task::Task;
use utils::log_init;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_init();
    let config = Config::from_file("")?;
    let mut runtime = Runtime::init(config)?;
    // Add and spawn tasks.
    let task = Task::from_path("")?;
    runtime.new_task(task);

    // TODO: Use models.
    Ok(())
}
