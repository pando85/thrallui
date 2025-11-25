use contracts::session::SessionInfoDTO;

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SessionListProps {
    pub sessions: Vec<SessionInfoDTO>,
    pub active_session_id: Option<String>,
    pub on_select: EventHandler<String>,
    pub on_close: EventHandler<String>,
}

#[component]
pub fn SessionList(props: SessionListProps) -> Element {
    rsx! {
        div { class: "session-list",

            h3 { "Active Sessions ({props.sessions.len()})" }

            if props.sessions.is_empty() {
                div { class: "no-sessions",
                    p { "No active sessions" }
                    p { class: "hint", "Create a new session above to get started" }
                }
            } else {
                div { class: "sessions",
                    for session in &props.sessions {
                        SessionItem {
                            key: "{session.id}",
                            session: session.clone(),
                            is_active: props.active_session_id.as_ref() == Some(&session.id),
                            on_select: move |id| props.on_select.call(id),
                            on_close: move |id| props.on_close.call(id),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SessionItemProps {
    session: SessionInfoDTO,
    is_active: bool,
    on_select: EventHandler<String>,
    on_close: EventHandler<String>,
}

#[component]
fn SessionItem(props: SessionItemProps) -> Element {
    let session_id = props.session.id.clone();
    let session_id_for_close = session_id.clone();

    let item_class = if props.is_active {
        "session-item active"
    } else {
        "session-item"
    };

    rsx! {
        div {
            class: "{item_class}",
            onclick: move |_| props.on_select.call(session_id.clone()),

            div { class: "session-info",
                div { class: "session-name", "{props.session.name}" }
                div { class: "session-directory", "{props.session.directory}" }
                div { class: "session-time", "{format_time(&props.session.created_at)}" }
            }

            button {
                class: "btn-close",
                onclick: move |evt| {
                    evt.stop_propagation();
                    props.on_close.call(session_id_for_close.clone());
                },
                "×"
            }
        }
    }
}

fn format_time(iso_time: &str) -> String {
    if let Some(time_part) = iso_time.split('T').nth(1) {
        if let Some(time) = time_part.split('.').next() {
            return time.to_string();
        }
    }
    iso_time.to_string()
}
