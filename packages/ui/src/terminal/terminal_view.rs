use dioxus::prelude::*;

#[cfg(target_family = "wasm")]
use web_sys;

#[derive(Props, Clone, PartialEq)]
pub struct TerminalViewProps {
    pub session_id: String,
    pub output: Vec<String>,
    pub on_send_input: EventHandler<String>,
}

#[component]
pub fn TerminalView(props: TerminalViewProps) -> Element {
    let mut input_value = use_signal(|| String::new());

    #[cfg(target_family = "wasm")]
    let mut output_ref = use_signal(|| None::<web_sys::Element>);

    // Auto-scroll to bottom when output changes
    #[cfg(target_family = "wasm")]
    use_effect(move || {
        if let Some(element) = output_ref.read().as_ref() {
            element.set_scroll_top(element.scroll_height());
        }
    });

    let handle_send_click = move |_evt: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
        let input = input_value.read().clone();
        if !input.is_empty() {
            props.on_send_input.call(input);
            input_value.set(String::new());
        }
    };

    let mut handle_send_keydown =
        move |_evt: dioxus::prelude::Event<dioxus::prelude::KeyboardData>| {
            let input = input_value.read().clone();
            if !input.is_empty() {
                props.on_send_input.call(input);
                input_value.set(String::new());
            }
        };

    rsx! {
        div {
            class: "terminal-view",

            div {
                class: "terminal-output",
                onmounted: move |_evt| {
                    #[cfg(target_family = "wasm")]
                    if let Some(element) = _evt.data().downcast::<web_sys::Element>() {
                        output_ref.set(Some(element.clone()));
                    }
                },

                for (idx, line) in props.output.iter().enumerate() {
                    div {
                        key: "{props.session_id}-{idx}",
                        class: "output-line",
                        dangerous_inner_html: "{escape_html(line)}"
                    }
                }

                if props.output.is_empty() {
                    div { class: "terminal-placeholder", "Waiting for output..." }
                }
            }

            div {
                class: "terminal-input-area",

                input {
                    r#type: "text",
                    class: "terminal-input",
                    value: "{input_value}",
                    placeholder: "Type a command and press Enter...",
                    oninput: move |evt| input_value.set(evt.value().clone()),
                    onkeydown: move |evt| {
                        if evt.key() == Key::Enter {
                            handle_send_keydown(evt);
                        }
                    },
                    autofocus: true,
                }

                button {
                    class: "btn btn-send",
                    onclick: handle_send_click,
                    "Send"
                }
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
