#[cfg(feature = "server")]
mod server;

use serde::{Deserialize, Serialize};

use dioxus::logger::tracing;
use dioxus::prelude::*;

/// Lightweight session information sent to clients
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionInfoDTO {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub created_at: String, // ISO 8601 timestamp
}

/// Create a new terminal session
#[post("/api/sessions/create")]
pub async fn create_session(
    name: String,
    directory: String,
    command: String,
) -> Result<SessionInfoDTO, ServerFnError> {
    use chrono::Utc;
    use uuid::Uuid;

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

    Ok(SessionInfoDTO {
        id: session_id,
        name,
        directory,
        created_at,
    })
}
