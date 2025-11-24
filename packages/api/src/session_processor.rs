use crate::config::Config;
use crate::models::session::{SessionConfig, SessionInfo};
use crate::session_handler::SessionManager;
use std::sync::{Arc, OnceLock};
use tokio::sync::mpsc;

#[cfg(feature = "server")]
pub enum SessionRequest {
    CreateSession {
        name: String,
        directory: String,
        response_tx: tokio::sync::oneshot::Sender<Result<String, String>>,
    },
    GetSessions {
        response_tx: tokio::sync::oneshot::Sender<Result<Vec<SessionInfo>, String>>,
    },
    DeleteSession {
        session_id: String,
        response_tx: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
}

#[cfg(feature = "server")]
pub static SESSION_REQUEST_TX: OnceLock<mpsc::Sender<SessionRequest>> = OnceLock::new();

#[cfg(feature = "server")]
pub struct SessionProcessor {
    config: Arc<Config>,
    request_rx: mpsc::Receiver<SessionRequest>,
}

#[cfg(feature = "server")]
impl SessionProcessor {
    pub fn new(config: Arc<Config>) -> (Self, mpsc::Sender<SessionRequest>) {
        let (request_tx, request_rx) = mpsc::channel(100);

        (Self { config, request_rx }, request_tx)
    }

    pub async fn run(mut self) {
        while let Some(request) = self.request_rx.recv().await {
            match request {
                SessionRequest::CreateSession {
                    name,
                    directory,
                    response_tx,
                } => {
                    let session_manager = SessionManager::new(Arc::clone(&self.config));
                    let session_config = SessionConfig { name, directory };
                    let result = session_manager
                        .create_session(session_config)
                        .await
                        .map_err(|e| e.to_string());
                    let _ = response_tx.send(result);
                }
                SessionRequest::GetSessions { response_tx } => {
                    let session_manager = SessionManager::new(Arc::clone(&self.config));
                    let result = session_manager
                        .list_sessions()
                        .await
                        .map_err(|e| e.to_string());
                    let _ = response_tx.send(result);
                }
                SessionRequest::DeleteSession {
                    session_id,
                    response_tx,
                } => {
                    let session_manager = SessionManager::new(Arc::clone(&self.config));
                    let result = session_manager
                        .close_session(&session_id)
                        .await
                        .map_err(|e| e.to_string());
                    let _ = response_tx.send(result);
                }
            }
        }
    }
}
