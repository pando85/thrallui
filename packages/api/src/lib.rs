mod config;
mod models;

pub use config::Config;
use dioxus::prelude::*;

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

/// Get all child directories (depth level 1) from the configured workspace root
#[get("/api/directories")]
pub async fn get_workspace_directories() -> Result<Vec<contracts::directory::DirectoryInfo>, ServerFnError> {
    use std::fs;

    let config = Config::get();
    let root_path = &config.workspace_root;

    // Read the workspace root directory
    let entries = fs::read_dir(root_path)
        .map_err(|e| ServerFnError::new(format!("Failed to read workspace directory: {}", e)))?;

    let mut directories = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| ServerFnError::new(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();

        // Only include directories (depth level 1)
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                directories.push(contracts::directory::DirectoryInfo {
                    name: name.to_string(),
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    // Sort by name for consistent ordering
    directories.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(directories)
}
