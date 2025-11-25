use serde::{Deserialize, Serialize};

/// Lightweight session information sent to clients
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionInfoDTO {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub created_at: String, // ISO 8601 timestamp
}
