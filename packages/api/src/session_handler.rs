use crate::config::Config;
use crate::models::session::{Session, SessionConfig, SessionInfo, SessionMetadata};
use crate::process_manager::ProcessManager;
use crate::state::app_state::SessionMetadataStore;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    process_manager: ProcessManager,
    config: Arc<Config>,
    metadata_store: Option<SessionMetadataStore>,
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            sessions: Arc::clone(&self.sessions),
            process_manager: ProcessManager::new(),
            config: Arc::clone(&self.config),
            metadata_store: self.metadata_store.clone(),
        }
    }
}

impl SessionManager {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            process_manager: ProcessManager::new(),
            config,
            metadata_store: None,
        }
    }

    pub fn set_metadata_store(&mut self, store: SessionMetadataStore) {
        self.metadata_store = Some(store);
    }

    pub async fn create_session(&self, session_config: SessionConfig) -> Result<String> {
        session_config.validate()?;

        let dir_path = std::path::PathBuf::from(&session_config.directory);
        if !self.config.is_directory_allowed(&dir_path) {
            anyhow::bail!("Directory not allowed: {}", session_config.directory);
        }

        let sessions = self.sessions.read().await;
        if sessions.len() >= self.config.max_sessions {
            anyhow::bail!("Maximum session limit reached");
        }
        drop(sessions);

        let session_id = Uuid::new_v4().to_string();
        let mut session = Session::new(
            session_id.clone(),
            session_config.name,
            session_config.directory.clone(),
        );

        let pty_pair = self
            .process_manager
            .spawn_claude(&session_config.directory, &self.config.claude_path)?;

        session.pty_pair = Some(Mutex::new(pty_pair));

        let mut sessions = self.sessions.write().await;
        let metadata = SessionMetadata::from(&session);
        sessions.insert(session_id.clone(), session);

        // Update metadata store for WebSocket access
        if let Some(ref store) = self.metadata_store {
            store.update_metadata(&session_id, metadata).await;
        }

        Ok(session_id)
    }

    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().map(|s| s.to_info()).collect())
    }

    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id).context("Session not found")?;

        // Remove from metadata store
        if let Some(ref store) = self.metadata_store {
            store.remove_metadata(session_id).await;
        }

        Ok(())
    }

    pub async fn session_exists(&self, session_id: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(session_id)
    }

    pub async fn get_session_metadata(&self, session_id: &str) -> Option<SessionMetadata> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(SessionMetadata::from)
    }

    pub async fn add_session_output(&self, session_id: &str, output: String) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id).context("Session not found")?;
        session.add_output(output);
        Ok(())
    }

    pub async fn get_session_output(&self, session_id: &str) -> Result<Vec<String>> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).context("Session not found")?;
        Ok(session.get_all_output())
    }

    pub async fn get_pty_reader(&self, session_id: &str) -> Result<Box<dyn std::io::Read + Send>> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).context("Session not found")?;
        let pty_pair = session.pty_pair.as_ref().context("No PTY")?;
        let pty_pair = pty_pair
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        pty_pair
            .master
            .try_clone_reader()
            .context("Failed to clone reader")
    }

    pub async fn get_pty_writer(&self, session_id: &str) -> Result<Box<dyn std::io::Write + Send>> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).context("Session not found")?;
        let pty_pair = session.pty_pair.as_ref().context("No PTY")?;
        let pty_pair = pty_pair
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        pty_pair
            .master
            .take_writer()
            .context("Failed to get writer")
    }
}
