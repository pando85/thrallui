mod config;
mod models;

pub use config::Config;

use dioxus::logger::tracing;
use dioxus::prelude::*;

#[cfg(feature = "server")]
use uuid::Uuid;

/// Get all child directories (depth level 1) from the configured workspace root
#[get("/api/directories")]
pub async fn get_workspace_directories(
) -> Result<Vec<contracts::directory::DirectoryInfo>, ServerFnError> {
    use std::fs;

    let config = Config::get();
    let root_path = &config.workspace_root;

    // Read the workspace root directory
    let entries = fs::read_dir(root_path)
        .map_err(|e| ServerFnError::new(format!("Failed to read workspace directory: {}", e)))?;

    let mut directories = Vec::new();

    for entry in entries {
        let entry = entry
            .map_err(|e| ServerFnError::new(format!("Failed to read directory entry: {}", e)))?;
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

/// Create a new terminal session
#[post("/api/sessions/create")]
pub async fn create_session(
    name: String,
    directory: String,
    command: String,
) -> Result<contracts::session::SessionInfoDTO, ServerFnError> {
    use chrono::Utc;

    // Generate a unique session ID
    let session_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    // TODO: Actually spawn a PTY process here
    // For now, we just return the session info
    tracing::info!(
        "Creating session: id={}, name={}, directory={}, command={}",
        session_id,
        name,
        directory,
        command
    );

    Ok(contracts::session::SessionInfoDTO {
        id: session_id,
        name,
        directory,
        created_at,
    })
}
