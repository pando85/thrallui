#[cfg(not(target_family = "wasm"))]
use chrono::{DateTime, Utc};
#[cfg(not(target_family = "wasm"))]
use portable_pty::PtyPair;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Full session state stored on the server
#[cfg(not(target_family = "wasm"))]
pub struct Session {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub created_at: DateTime<Utc>,
    pub pty_pair: Option<Mutex<PtyPair>>,
    pub output_buffer: Vec<String>,
}

#[cfg(not(target_family = "wasm"))]
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("directory", &self.directory)
            .field("created_at", &self.created_at)
            .field("pty_pair", &self.pty_pair.is_some())
            .field("output_buffer", &self.output_buffer)
            .finish()
    }
}

/// Session metadata that can be safely shared across threads
#[cfg(not(target_family = "wasm"))]
#[derive(Clone, Debug)]
pub struct SessionMetadata {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(not(target_family = "wasm"))]
impl From<&Session> for SessionMetadata {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id.clone(),
            name: session.name.clone(),
            directory: session.directory.clone(),
            created_at: session.created_at,
        }
    }
}

#[cfg(not(target_family = "wasm"))]
impl Session {
    pub fn new(id: String, name: String, directory: String) -> Self {
        Self {
            id,
            name,
            directory,
            created_at: Utc::now(),
            pty_pair: None,
            output_buffer: Vec::new(),
        }
    }

    pub fn to_info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            directory: self.directory.clone(),
            created_at: self.created_at.to_rfc3339(),
        }
    }

    pub fn add_output(&mut self, output: String) {
        self.output_buffer.push(output);
    }

    pub fn get_all_output(&self) -> Vec<String> {
        self.output_buffer.clone()
    }
}

/// Lightweight session information sent to clients
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub created_at: String, // ISO 8601 timestamp
}

/// Configuration for creating new sessions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionConfig {
    pub name: String,
    pub directory: String,
}

impl SessionConfig {
    #[cfg(not(target_family = "wasm"))]
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.name.trim().is_empty() {
            anyhow::bail!("Session name cannot be empty");
        }
        if self.name.len() > 100 {
            anyhow::bail!("Session name too long (max 100 characters)");
        }
        if self.directory.trim().is_empty() {
            anyhow::bail!("Directory cannot be empty");
        }
        // Check if directory exists and is accessible
        let path = std::path::Path::new(&self.directory);
        if !path.exists() {
            anyhow::bail!("Directory does not exist: {}", self.directory);
        }
        if !path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", self.directory);
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "test-id".to_string(),
            "Test Session".to_string(),
            "/tmp".to_string(),
        );

        assert_eq!(session.id, "test-id");
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.directory, "/tmp");
        assert!(session.pty_pair.is_none());
        assert!(session.output_buffer.is_empty());
    }

    #[test]
    fn test_session_to_info() {
        let session = Session::new(
            "test-id".to_string(),
            "Test Session".to_string(),
            "/tmp".to_string(),
        );

        let info = session.to_info();
        assert_eq!(info.id, "test-id");
        assert_eq!(info.name, "Test Session");
        assert_eq!(info.directory, "/tmp");
        // created_at should be a valid RFC3339 timestamp
        assert!(chrono::DateTime::parse_from_rfc3339(&info.created_at).is_ok());
    }

    #[test]
    fn test_session_info_serialization() {
        let info = SessionInfo {
            id: "test-id".to_string(),
            name: "Test Session".to_string(),
            directory: "/tmp".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: SessionInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info, deserialized);
    }

    #[test]
    fn test_session_config_serialization() {
        let config = SessionConfig {
            name: "Test Session".to_string(),
            directory: "/tmp".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SessionConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }
}
