use api::models::message::{ClientEvent, ServerEvent, SessionInfo};
use api::{create_session, delete_session, get_allowed_directories, get_sessions};
use dioxus::prelude::*;
use futures::channel::mpsc;
use futures::{select, FutureExt};
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use ui::terminal::{SessionList, SessionManager, TerminalView};

/// Terminal view route
#[component]
pub fn Terminal() -> Element {
    // Application state
    let sessions = use_signal(|| Vec::<SessionInfo>::new());
    let mut active_session_id = use_signal(|| Option::<String>::None);
    let mut terminal_outputs = use_signal(|| HashMap::<String, Vec<String>>::new());
    let connection_status = use_signal(|| "Connecting...".to_string());
    let mut allowed_directories = use_signal(|| Vec::<String>::new());

    // Fetch allowed directories on mount
    use_effect(move || {
        spawn(async move {
            match get_allowed_directories().await {
                Ok(dirs) => {
                    allowed_directories.set(dirs);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch allowed directories: {:?}", e);
                }
            }
        });
    });

    // Channel for sending messages to WebSocket
    let tx = use_signal(|| None::<Arc<Mutex<mpsc::Sender<ClientEvent>>>>);

    // Establish WebSocket connection
    use_effect(move || {
        let mut sessions_clone = sessions.clone();
        let _active_session_id_clone = active_session_id.clone();
        let mut terminal_outputs_clone = terminal_outputs.clone();
        let mut connection_status_clone = connection_status.clone();
        let mut tx_signal = tx.clone();

        spawn(async move {
            // Create channel for this WebSocket connection
            let (tx_ws, mut rx) = mpsc::channel::<ClientEvent>(100);

            // Store the sender so other parts can use it
            tx_signal.set(Some(Arc::new(Mutex::new(tx_ws))));

            // Load initial sessions from REST API
            match get_sessions().await {
                Ok(session_list) => {
                    sessions_clone.set(session_list);
                }
                Err(e) => {
                    tracing::error!("Failed to load sessions: {:?}", e);
                }
            }

            // Build WebSocket URL dynamically based on current location
            #[cfg(target_family = "wasm")]
            let ws_url = {
                use gloo_net::websocket::Message;
                use web_sys::window as web_window;

                let window = web_window().expect("no global window exists");
                let location = window.location();
                let host = location.host().unwrap_or_else(|_| "localhost:8080".to_string());
                let protocol = if location.protocol().unwrap_or_default() == "https:" {
                    "wss:"
                } else {
                    "ws:"
                };
                format!("{}//{}/api/terminal", protocol, host)
            };

            #[cfg(not(target_family = "wasm"))]
            let ws_url = "ws://localhost:8080/api/terminal".to_string();

            tracing::info!("Connecting to WebSocket: {}", ws_url);

            // Connect to WebSocket for terminal I/O
            match WebSocket::open(&ws_url) {
                Ok(mut socket) => {
                    tracing::info!("WebSocket connected successfully");
                    connection_status_clone.set("Connected".to_string());

                    loop {
                        select! {
                            // Handle incoming WebSocket messages
                            msg = socket.next().fuse() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        tracing::debug!("Received WebSocket message: {}", text);
                                        match serde_json::from_str::<ServerEvent>(&text) {
                                            Ok(event) => {
                                                tracing::debug!("Parsed server event: {:?}", event);
                                                handle_server_event(event, &mut terminal_outputs_clone);
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to parse server event: {}", e);
                                            }
                                        }
                                    }
                                    Some(Ok(Message::Bytes(_))) => {}
                                    Some(Err(e)) => {
                                        tracing::error!("WebSocket error: {:?}", e);
                                        connection_status_clone.set("Error".to_string());
                                        break;
                                    }
                                    None => break,
                                }
                            }

                            // Handle outgoing messages from channel
                            event = rx.next().fuse() => {
                                match event {
                                    Some(event) => {
                                        tracing::debug!("Sending client event via WebSocket: {:?}", event);
                                        match serde_json::to_string(&event) {
                                            Ok(json) => {
                                                if let Err(e) = socket.send(Message::Text(json)).await {
                                                    tracing::error!("Failed to send WebSocket message: {:?}", e);
                                                    break;
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to serialize event: {:?}", e);
                                            }
                                        }
                                    }
                                    None => {
                                        tracing::warn!("Client event channel closed");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to connect to WebSocket: {:?}", e);
                    connection_status_clone.set("Failed to connect".to_string());
                }
            }
        });
    });

    // Event handlers
    let handle_create_session = Rc::new(RefCell::new(move |name: String, directory: String, command: String| {
        let mut sessions = sessions.clone();
        let mut active_session_id = active_session_id.clone();
        let tx_clone = tx.clone();
        spawn(async move {
            match create_session(name.clone(), directory.clone()).await {
                Ok(session_id) => {
                    tracing::info!("Session created: {}", session_id);

                    // Add to local session list
                    let mut current_sessions = sessions.read().clone();
                    current_sessions.push(SessionInfo {
                        id: session_id.clone(),
                        name,
                        directory,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    });
                    sessions.set(current_sessions);

                    // Automatically select the newly created session
                    active_session_id.set(Some(session_id.clone()));

                    // Wait for session to be fully initialized on the server
                    // TODO: Replace with a proper "session ready" event from server
                    #[cfg(target_family = "wasm")]
                    {
                        use gloo_timers::future::TimeoutFuture;
                        TimeoutFuture::new(500).await;
                    }

                    #[cfg(not(target_family = "wasm"))]
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    // Send the command to the session after it's ready
                    if !command.is_empty() {
                        tracing::info!("Sending command to session {}: {}", session_id, command);
                        if let Some(tx_arc) = tx_clone.read().as_ref() {
                            if let Ok(mut tx_guard) = tx_arc.lock() {
                                let _ = tx_guard.send(ClientEvent::SendInput {
                                    session_id,
                                    input: format!("{}\n", command)
                                }).await;
                            }
                        } else {
                            tracing::warn!("WebSocket sender not ready");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create session: {:?}", e);
                }
            }
        });
    }));

    let handle_select_session = Rc::new(RefCell::new(move |session_id: String| {
        active_session_id.set(Some(session_id.clone()));

        // Request buffered history for this session via WebSocket
        let tx_clone = tx.clone();
        spawn(async move {
            if let Some(tx_arc) = tx_clone.read().as_ref() {
                if let Ok(mut tx_guard) = tx_arc.lock() {
                    let _ = tx_guard.send(ClientEvent::RequestHistory { session_id }).await;
                }
            } else {
                tracing::warn!("WebSocket sender not ready");
            }
        });
    }));

    let handle_close_session = Rc::new(RefCell::new(move |session_id: String| {
        let mut sessions = sessions.clone();
        let session_id_clone = session_id.clone();
        spawn(async move {
            match delete_session(session_id_clone.clone()).await {
                Ok(_) => {
                    // Remove from local session list
                    let mut current_sessions = sessions.read().clone();
                    current_sessions.retain(|s| s.id != session_id_clone);
                    sessions.set(current_sessions);

                    // Clear from local state
                    terminal_outputs.write().remove(&session_id_clone);
                    if active_session_id.read().as_ref() == Some(&session_id_clone) {
                        active_session_id.set(None);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to delete session: {:?}", e);
                }
            }
        });
    }));

    let handle_send_input = Rc::new(RefCell::new(move |input: String| {
        if let Some(ref session_id) = *active_session_id.read() {
            let tx_clone = tx.clone();
            let session_id = session_id.clone();
            spawn(async move {
                if let Some(tx_arc) = tx_clone.read().as_ref() {
                    if let Ok(mut tx_guard) = tx_arc.lock() {
                        let _ = tx_guard.send(ClientEvent::SendInput { session_id, input }).await;
                    }
                } else {
                    tracing::warn!("WebSocket sender not ready");
                }
            });
        }
    }));

    rsx! {
        div { class: "terminal-container",

            // Left panel: Session management
            div { class: "terminal-left-panel",

                h2 { "Terminal Sessions" }

                SessionManager {
                    on_create: move |(name, directory, command)| handle_create_session.borrow_mut()(name, directory, command),
                    allowed_directories: allowed_directories.read().clone(),
                }

                SessionList {
                    sessions: sessions.read().clone(),
                    active_session_id: active_session_id.read().clone(),
                    on_select: move |id| handle_select_session.borrow_mut()(id),
                    on_close: move |id| handle_close_session.borrow_mut()(id),
                }
            }

            // Right panel: Terminal view
            div { class: "terminal-right-panel",

                if let Some(ref session_id) = *active_session_id.read() {
                    TerminalView {
                        session_id: session_id.clone(),
                        output: terminal_outputs.read().get(session_id).cloned().unwrap_or_default(),
                        on_send_input: move |input| handle_send_input.borrow_mut()(input),
                    }
                } else {
                    div { class: "no-session-selected",
                        h3 { "No session selected" }
                        p { "Create a new session or select one from the left panel" }
                    }
                }
            }

            // Status bar
            div { class: "status-bar", "Status: {connection_status}" }
        }
    }
}

/// Handle incoming server events and update state
fn handle_server_event(
    event: ServerEvent,
    terminal_outputs: &mut Signal<HashMap<String, Vec<String>>>,
) {
    match event {
        ServerEvent::TerminalOutput { session_id, data } => {
            tracing::debug!("Terminal output for session {}: {}", session_id, data);
            terminal_outputs
                .write()
                .entry(session_id)
                .or_insert_with(Vec::new)
                .push(data);
        }

        ServerEvent::Error { message } => {
            tracing::error!("Server error: {}", message);
            // TODO: Show error to user in UI
        }

        // Session management events are not expected on this WebSocket
        ServerEvent::SessionList { .. }
        | ServerEvent::SessionCreated { .. }
        | ServerEvent::SessionClosed { .. } => {
            tracing::warn!(
                "Received unexpected session management event - these should use REST API"
            );
        }
    }
}
