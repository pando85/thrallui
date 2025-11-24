use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SessionManagerProps {
    pub on_create: EventHandler<(String, String, String)>, // (name, directory, command)
    pub allowed_directories: Vec<String>,
}

#[component]
pub fn SessionManager(props: SessionManagerProps) -> Element {
    let mut task_description = use_signal(|| String::new());
    let mut selected_directory = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);

    // Reactively update selected directory when directories become available or change
    let allowed_directories = props.allowed_directories.clone();
    use_effect(use_reactive!(|allowed_directories| {
        if !allowed_directories.is_empty() && selected_directory().is_empty() {
            selected_directory.set(allowed_directories[0].clone());
        }
    }));

    let mut create_session = move || {
        let task = task_description();
        let dir = selected_directory();

        if task.is_empty() {
            error_message.set(Some("Task description is required".to_string()));
            return;
        }

        if dir.is_empty() {
            error_message.set(Some("Please select a directory".to_string()));
            return;
        }

        error_message.set(None);

        // Generate a session name from task (first 50 chars)
        let session_name = if task.len() > 50 {
            format!("{}...", &task[..50])
        } else {
            task.clone()
        };

        // Pass name, directory, and the full command/task
        props.on_create.call((session_name, dir.clone(), task.clone()));

        // Clear task description after creating
        task_description.set(String::new());
    };

    let mut handle_create = move |_| create_session();
    let handle_button_click = move |_| create_session();

    rsx! {
        div { class: "session-manager",

            if let Some(ref error) = *error_message.read() {
                div { class: "error-message", "{error}" }
            }

            // Full-width task description input (headless - no label)
            div { class: "form-group-headless",
                div { class: "input-container",
                    input {
                        r#type: "text",
                        class: "task-input",
                        value: "{task_description}",
                        placeholder: "Find a small pending task in the code and do it",
                        oninput: move |evt| task_description.set(evt.value().clone()),
                        onkeydown: move |evt| {
                            if evt.key() == Key::Enter {
                                handle_create(evt);
                            }
                        },
                    }
                    button { class: "send-button", onclick: handle_button_click, "↑" }
                }
            }

            // Directory selector dropdown (headless - no label)
            div { class: "form-group-headless",
                select {
                    class: "directory-select",
                    value: selected_directory(),
                    onchange: move |evt| selected_directory.set(evt.value().clone()),

                    for directory in &allowed_directories {
                        option { value: "{directory}", "{directory}" }
                    }
                }
            }
        }
    }
}
