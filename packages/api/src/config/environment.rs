use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub claude_path: String,
    pub max_sessions: usize,
    pub allowed_directories: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("THRALLUI_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("THRALLUI_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            claude_path: env::var("THRALLUI_CLAUDE_PATH").unwrap_or_else(|_| "claude".to_string()),
            max_sessions: env::var("THRALLUI_MAX_SESSIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            allowed_directories: vec![
                env::var("THRALLUI_ALLOWED_DIRS").unwrap_or_else(|_| "/tmp".to_string())
            ],
        }
    }

    pub fn is_directory_allowed(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();
        self.allowed_directories
            .iter()
            .any(|allowed| path_str.starts_with(allowed) || allowed == "*")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            claude_path: "claude".to_string(),
            max_sessions: 10,
            allowed_directories: vec!["/home".to_string(), "/tmp".to_string()],
        }
    }
}
