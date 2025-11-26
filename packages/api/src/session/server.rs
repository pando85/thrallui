use super::SessionInfoDTO;

use chrono::{DateTime, Utc};

/// Full session state stored on the server
#[derive(Debug)]
struct Session {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(not(target_family = "wasm"))]
impl Session {
    pub fn new(id: String, name: String, directory: String) -> Self {
        Self {
            id,
            name,
            directory,
            created_at: Utc::now(),
        }
    }

    pub fn to_info(&self) -> SessionInfoDTO {
        SessionInfoDTO {
            id: self.id.clone(),
            name: self.name.clone(),
            directory: self.directory.clone(),
            created_at: self.created_at.to_rfc3339(),
        }
    }
}
