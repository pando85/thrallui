use serde::{Deserialize, Serialize};

/// Events sent FROM client TO server via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ClientEvent {
    /// Create a new terminal session
    CreateSession { name: String, directory: String },

    /// Send input to a terminal session
    SendInput { session_id: String, input: String },

    /// Request session history (buffered output)
    RequestHistory { session_id: String },

    /// Close a session
    CloseSession { session_id: String },
}

/// Events sent FROM server TO client via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ServerEvent {
    /// Session was created successfully
    SessionCreated {
        session_id: String,
        name: String,
        directory: String,
    },

    /// List of all active sessions (sent on connect)
    SessionList { sessions: Vec<SessionInfo> },

    /// Terminal output data
    TerminalOutput { session_id: String, data: String },

    /// Session was closed
    SessionClosed {
        session_id: String,
        reason: Option<String>,
    },

    /// Error occurred
    Error { message: String },
}

/// Lightweight session information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub created_at: String, // ISO 8601 timestamp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_event_serialization() {
        let event = ClientEvent::CreateSession {
            name: "test".into(),
            directory: "/tmp".into(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ClientEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_server_event_serialization() {
        let event = ServerEvent::TerminalOutput {
            session_id: "123".into(),
            data: "Hello\n".into(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ServerEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
    }
}
