use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TerminalViewProps {
    pub session_id: String,
    pub output: Vec<String>,
    pub on_send_input: EventHandler<String>,
}

#[component]
pub fn TerminalView(props: TerminalViewProps) -> Element {
    let mut input_value = use_signal(|| String::new());
    let handle_send_click = move |_| {
        let input = input_value.read().clone();
        if !input.is_empty() {
            props.on_send_input.call(input.clone());
            input_value.set(String::new());
        }
    };
    rsx! {
        div { class: "terminal-view",

            div { class: "terminal-output", onmounted: move |_evt| {},

                for (idx , line) in props.output.iter().enumerate() {
                    div {
                        key: "{props.session_id}-{idx}",
                        class: "output-line",
                        dangerous_inner_html: "{escape_html(line)}",
                    }
                }

                if props.output.is_empty() {
                    div { class: "terminal-placeholder", "Waiting for output..." }
                }
            }

            div { class: "terminal-input-area",

                input {
                    r#type: "text",
                    class: "terminal-input",
                    value: "{input_value}",
                    placeholder: "Type a command and press Enter...",
                    oninput: move |evt| input_value.set(evt.value().clone()),
                    autofocus: true,
                }

                button { class: "btn-send", onclick: handle_send_click, "Send" }
            }
        }
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
