use crate::config::Config;
use crate::models::session::SessionMetadata;
use crate::session_handler::SessionManager;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

/// Global app state instance
static GLOBAL_APP_STATE: OnceLock<AppState> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct AppState {
    pub session_manager: SessionManager,
    pub config: Arc<Config>,
    pub session_metadata_store: SessionMetadataStore,
}

/// Thread-safe session metadata store for WebSocket access
#[derive(Clone, Debug)]
pub struct SessionMetadataStore {
    metadata: Arc<RwLock<HashMap<String, SessionMetadata>>>,
}

impl SessionMetadataStore {
    pub fn new() -> Self {
        Self {
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn session_exists(&self, session_id: &str) -> bool {
        let metadata = self.metadata.read().await;
        metadata.contains_key(session_id)
    }

    pub async fn update_metadata(&self, session_id: &str, metadata: SessionMetadata) {
        let mut store = self.metadata.write().await;
        store.insert(session_id.to_string(), metadata);
    }

    pub async fn remove_metadata(&self, session_id: &str) {
        let mut store = self.metadata.write().await;
        store.remove(session_id);
    }
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let mut session_manager = SessionManager::new(Arc::clone(&config));
        let session_metadata_store = SessionMetadataStore::new();

        // Set the metadata store on the session manager
        session_manager.set_metadata_store(session_metadata_store.clone());

        Self {
            session_manager,
            config,
            session_metadata_store,
        }
    }

    /// Initialize the global app state (call once on server startup)
    pub fn init(config: Config) {
        let state = Self::new(config);
        GLOBAL_APP_STATE.set(state).ok();
    }

    /// Get global app state (for use in server functions and WebSocket handlers)
    pub fn global() -> AppState {
        GLOBAL_APP_STATE
            .get()
            .expect("AppState not initialized. Call AppState::init() on server startup.")
            .clone()
    }
}
