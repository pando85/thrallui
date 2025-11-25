use clap::Parser;
use std::sync::OnceLock;

/// Application configuration
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Root workspace directory for terminal sessions
    #[arg(
        long,
        env = "THRALLUI_WORKSPACE_ROOT",
        default_value = "/tmp/workspace"
    )]
    pub workspace_root: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    /// Initialize the global config from command-line arguments and environment variables
    pub fn init() -> &'static Config {
        CONFIG.get_or_init(|| Config::parse())
    }

    /// Get the initialized config (panics if not initialized)
    pub fn get() -> &'static Config {
        CONFIG
            .get()
            .expect("Config not initialized. Call Config::init() first.")
    }
}
