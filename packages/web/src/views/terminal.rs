use api::directory::get_workspace_directories;
use api::session::{close_session, create_session, SessionInfoDTO};
use ui::terminal::{SessionList, SessionManager, TerminalView};

use dioxus::logger::tracing;
use dioxus::prelude::*;

#[component]
pub fn Terminal() -> Element {
    let connection_status = "mocked as connected";
    let mut sessions = use_signal(|| Vec::<SessionInfoDTO>::new());
    let mut active_session_id = use_signal(|| Option::<String>::None);

    let handle_create_session = move |name: String, directory: String, command: String| async move {
        tracing::trace!("Creating session: {}, {}, {}", name, directory, command);

        match create_session(name, directory, command).await {
            Ok(session_info) => {
                let session_id = session_info.id.clone();

                // Add the new session to the list
                sessions.write().push(session_info);

                // Set it as the active session
                active_session_id.write().replace(session_id);
            }
            Err(e) => {
                tracing::error!("Failed to create session: {:?}", e);
            }
        }
    };
    let handle_select_session = move |id: String| {
        // Client-side only: update which session is active in the UI
        active_session_id.write().replace(id);
    };
    let handle_close_session = move |id: String| async move {
        tracing::trace!("Closing session: {}", id);

        // Call server function to terminate the session
        match close_session(id.clone()).await {
            Ok(_) => {
                // Remove from UI list
                sessions.retain(|s| s.id != id);

                // Clear active session if it was the one being closed
                if active_session_id.read().as_ref() == Some(&id) {
                    active_session_id.write().take();
                }
            }
            Err(e) => {
                tracing::error!("Failed to close session: {:?}", e);
            }
        }
    };
    // Fetch workspace directories from server
    let directories_future =
        use_server_future(move || async move { get_workspace_directories().await })?;

    let allowed_directories = use_memo(move || {
        directories_future()
            .and_then(|result| result.ok())
            .map(|dirs| dirs.into_iter().map(|d| d.path).collect())
            .unwrap_or_else(Vec::new)
    });

    let mut terminal_outputs =
        use_signal(|| std::collections::HashMap::<String, Vec<String>>::new());
    let handle_send_input = move |input: String| {
        if let Some(session_id) = active_session_id.read().as_ref() {
            terminal_outputs
                .write()
                .entry(session_id.clone())
                .or_default()
                .push(input);
        }
    };

    rsx! {
        div { class: "terminal-container",

            // Left panel: Session management
            div { class: "terminal-left-panel",

                h2 { class: "text-xl mb-4", "Terminal Sessions" }

                SessionManager {
                    on_create: move |(name, directory, command)| handle_create_session(name, directory, command),
                    allowed_directories: allowed_directories(),
                }

                SessionList {
                    sessions: sessions.read().clone(),
                    active_session_id: active_session_id.read().clone(),
                    // TODO: not need it? replace mutating active_session_id directly
                    on_select: handle_select_session,
                    on_close: handle_close_session,
                }
            }

            // Right panel: Terminal view
            div { class: "terminal-right-panel",

                if let Some(ref session_id) = *active_session_id.read() {
                    TerminalView {
                        session_id: session_id.clone(),
                        output: terminal_outputs.read().get(session_id).cloned().unwrap_or_default(),
                        on_send_input: handle_send_input,
                    }
                } else {
                    div { class: "no-session-selected",
                        h3 { class: "text-lg", "No session selected" }
                        p { "Create a new session or select one from the left panel" }
                    }
                }
            }

            // Status bar
            div { class: "status-bar", "Status: {connection_status}" }
        }
    }
}
