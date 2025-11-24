use crate::models::message::{ClientEvent, ServerEvent};
use crate::state::app_state::SessionMetadataStore;
use crate::AppState;
use dioxus::fullstack::{TypedWebsocket, WebSocketOptions, Websocket};
use dioxus::prelude::*;

/// Terminal WebSocket server function
///
/// This WebSocket endpoint handles real-time terminal I/O streaming.
/// Session management is handled via REST API.
#[get("/api/terminal")]
pub async fn terminal_websocket(
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent>, ServerFnError> {
    tracing::info!("WebSocket upgrade requested");

    Ok(options.on_upgrade(move |mut socket| async move {
        tracing::info!("WebSocket client connected for terminal I/O");

        // Get the global app state
        let app_state = match std::panic::catch_unwind(|| AppState::global()) {
            Ok(state) => state,
            Err(_) => {
                tracing::error!("Failed to get AppState - not initialized");
                let _ = socket
                    .send(ServerEvent::Error {
                        message: "Server state not initialized".to_string(),
                    })
                    .await;
                return;
            }
        };

        // Handle incoming terminal I/O messages from client
        while let Ok(event) = socket.recv().await {
            tracing::debug!("Received client event: {:?}", event);
            match handle_client_event(event, &mut socket, &app_state.session_metadata_store).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Error handling client event: {}", e);
                    let _ = socket
                        .send(ServerEvent::Error {
                            message: e.to_string(),
                        })
                        .await;
                }
            }
        }

        tracing::info!("WebSocket client disconnected");
    }))
}

/// Handle a single client event (terminal I/O only)
async fn handle_client_event(
    event: ClientEvent,
    socket: &mut TypedWebsocket<ClientEvent, ServerEvent>,
    metadata_store: &SessionMetadataStore,
) -> anyhow::Result<()> {
    match event {
        ClientEvent::SendInput { session_id, input } => {
            tracing::info!("Sending input to session {}", session_id);
            handle_send_input(session_id, input, socket, metadata_store).await?;
        }

        ClientEvent::RequestHistory { session_id } => {
            tracing::info!("Requesting history for session {}", session_id);
            handle_request_history(session_id, socket, metadata_store).await?;
        }

        // Session management must use REST API
        ClientEvent::CreateSession { .. } | ClientEvent::CloseSession { .. } => {
            tracing::warn!("Rejected session management event on WebSocket - use REST API");
            socket
                .send(ServerEvent::Error {
                    message: "Session management must use REST API, not WebSocket".to_string(),
                })
                .await?;
        }
    }

    Ok(())
}

/// Handle sending input to a session's PTY
async fn handle_send_input(
    session_id: String,
    input: String,
    socket: &mut TypedWebsocket<ClientEvent, ServerEvent>,
    metadata_store: &SessionMetadataStore,
) -> anyhow::Result<()> {
    // Check if session exists
    if !metadata_store.session_exists(&session_id).await {
        tracing::warn!("Input for non-existent session: {}", session_id);
        socket
            .send(ServerEvent::Error {
                message: format!("Session {} not found", session_id),
            })
            .await?;
        return Ok(());
    }

    // TODO: Send input to the actual PTY process
    // This requires:
    // 1. Get the PTY handle from session_manager
    // 2. Write input to PTY's stdin
    // 3. PTY output will be streamed back via a separate task

    // Mock implementation for testing
    tracing::debug!("Mock echo for session {}: {}", session_id, input.trim());
    socket
        .send(ServerEvent::TerminalOutput {
            session_id,
            data: format!("> {}\n", input.trim()),
        })
        .await?;

    Ok(())
}

/// Handle requesting buffered session history
async fn handle_request_history(
    session_id: String,
    socket: &mut TypedWebsocket<ClientEvent, ServerEvent>,
    metadata_store: &SessionMetadataStore,
) -> anyhow::Result<()> {
    // Check if session exists
    if !metadata_store.session_exists(&session_id).await {
        tracing::warn!("History requested for non-existent session: {}", session_id);
        socket
            .send(ServerEvent::Error {
                message: format!("Session {} not found", session_id),
            })
            .await?;
        return Ok(());
    }

    // TODO: Retrieve buffered history from session
    // This requires:
    // 1. Get the session from session_manager
    // 2. Retrieve buffered output (scrollback buffer)
    // 3. Send all buffered lines as TerminalOutput events

    // Mock implementation for testing
    tracing::debug!("Mock history for session {}", session_id);
    socket
        .send(ServerEvent::TerminalOutput {
            session_id,
            data: "Session started...\n".to_string(),
        })
        .await?;

    Ok(())
}
