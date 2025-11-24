//! API package - Backend server functions and WebSocket handlers

// Only compile these modules with server feature
#[cfg(feature = "server")]
pub mod config;
#[cfg(feature = "server")]
pub mod process_manager;
#[cfg(feature = "server")]
pub mod session_handler;
#[cfg(feature = "server")]
pub mod session_processor;
#[cfg(feature = "server")]
pub mod state;
#[cfg(feature = "server")]
pub mod websocket;

// Models are shared between client and server
pub mod models;

// Re-export commonly used items
pub use models::{
    message::{ClientEvent, ServerEvent},
    session::{SessionConfig, SessionInfo},
};

#[cfg(feature = "server")]
pub use models::session::Session;

#[cfg(feature = "server")]
pub use config::Config;
#[cfg(feature = "server")]
pub use state::AppState;
#[cfg(feature = "server")]
pub use websocket::terminal_websocket;

// Server functions
use dioxus::prelude::*;

#[post("/api/echo")]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

/// Get list of allowed directories from configuration
#[server]
pub async fn get_allowed_directories() -> Result<Vec<String>, ServerFnError> {
    use crate::config::Config;
    use std::fs;

    let config = Config::from_env();
    let base_dir = &config.allowed_directories[0];

    match fs::read_dir(base_dir) {
        Ok(entries) => {
            let mut children = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(path_str) = entry.path().to_str() {
                        children.push(path_str.to_string());
                    }
                }
            }
            Ok(children)
        }
        Err(e) => {
            tracing::warn!("Failed to read directory {}: {}", base_dir, e);
            Ok(vec![])
        }
    }
}

/// Get list of active sessions
#[server]
pub async fn get_sessions() -> Result<Vec<crate::models::message::SessionInfo>, ServerFnError> {
    let app_state = crate::AppState::global();

    match app_state.session_manager.list_sessions().await {
        Ok(sessions) => {
            // Convert from session::SessionInfo to message::SessionInfo
            Ok(sessions
                .into_iter()
                .map(|s| crate::models::message::SessionInfo {
                    id: s.id,
                    name: s.name,
                    directory: s.directory,
                    created_at: s.created_at,
                })
                .collect())
        }
        Err(e) => Err(ServerFnError::new(e.to_string())),
    }
}

/// Create a new session
#[server]
pub async fn create_session(name: String, directory: String) -> Result<String, ServerFnError> {
    use crate::models::session::SessionConfig;

    let app_state = crate::AppState::global();
    let session_config = SessionConfig { name, directory };

    match app_state.session_manager.create_session(session_config).await {
        Ok(session_id) => {
            tracing::info!("Session created via REST API: {}", session_id);
            Ok(session_id)
        }
        Err(e) => Err(ServerFnError::new(e.to_string())),
    }
}

/// Delete a session
#[server]
pub async fn delete_session(session_id: String) -> Result<(), ServerFnError> {
    let app_state = crate::AppState::global();

    match app_state.session_manager.close_session(&session_id).await {
        Ok(()) => {
            tracing::info!("Session deleted via REST API: {}", session_id);
            Ok(())
        }
        Err(e) => Err(ServerFnError::new(e.to_string())),
    }
}
