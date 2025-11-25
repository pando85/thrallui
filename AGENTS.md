# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ThrallUI-NG is a terminal management web application built with Dioxus 0.7. The project follows a workspace structure designed for multi-platform support, but **currently focuses exclusively on the web platform**. Desktop and mobile platforms exist in the workspace but are not actively developed.

## Workspace Structure

```
packages/
├── web/        # Web platform - ACTIVE DEVELOPMENT (work here)
├── desktop/    # Desktop platform - INACTIVE (ignore)
├── mobile/     # Mobile platform - INACTIVE (ignore)
├── ui/         # Shared UI components (platform-agnostic)
├── api/        # Server-side logic and server functions
└── contracts/  # Shared data types and DTOs
```

### Package Responsibilities

- **web**: The active web application - main entry point, routes, and web-specific components. **This is where you'll work most of the time.**
- **ui**: Shared components like `Navbar`, `SessionList`, `SessionManager`, `TerminalView`. Kept platform-agnostic for potential future use.
- **api**: Server functions using `#[post]` and `#[get]` macros. Only runs on server with `server` feature enabled.
- **contracts**: Shared types and DTOs for client-server communication (e.g., `SessionInfoDTO`, `ClientEvent`, `ServerEvent`).
- **desktop/mobile**: Inactive - ignore these packages.

## Build Commands

### Development

Navigate to the web package and start the dev server:
```bash
cd packages/web
dx serve
```

This starts the development server with hot reloading for the web application.

### Building

From the workspace root:
```bash
cargo build
```

Build the web application with features:
```bash
# Web (WASM) with server
cargo build --features server,web

# Web only
cargo build --features web
```

### Testing

Run tests from workspace root:
```bash
cargo test
```

Run tests for a specific package:
```bash
cargo test -p contracts
cargo test -p api
cargo test -p ui
```

## Terminal Architecture

The application implements a terminal management system with WebSocket communication:

### Message Flow
1. **Client Events** (`ClientEvent`): Commands sent from UI to server
   - `CreateSession`, `SendInput`, `RequestHistory`, `CloseSession`

2. **Server Events** (`ServerEvent`): Updates sent from server to UI
   - `SessionCreated`, `SessionList`, `TerminalOutput`, `SessionClosed`, `Error`

### Component Hierarchy
```
TerminalView (main view)
├── SessionManager (manages WebSocket & state)
└── SessionList (displays active sessions)
```

The `SessionManager` component handles WebSocket lifecycle and message routing. Terminal sessions are managed server-side with output streamed via WebSocket.

## Feature Flags

The web crate uses feature flags to control compilation:
- `web`: Web/WASM target (client-side)
- `server`: Server-side code (server functions, backend logic)

The `ui` crate's `server` feature cascades to `api/server`.

## Code Organization Principles

- **Shared UI components**: Put in `packages/ui/` (e.g., `SessionList`, `TerminalView`, `Navbar`)
- **Server logic**: Put in `packages/api/` (server functions)
- **Shared types/DTOs**: Put in `packages/contracts/` (used by both client and server)
- **Web-specific UI**: Put in `packages/web/src/views/` directory
- **Web routes**: Define in `packages/web/src/main.rs`

## Asset Management

Assets use the `asset!()` macro with paths relative to project root:
```rust
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
```

Web assets are located in `packages/web/assets/` directory.

---

# Dioxus 0.7 Reference

You are an expert [0.7 Dioxus](https://dioxuslabs.com/learn/0.7) assistant. Dioxus 0.7 changes every API in dioxus. Only use this up to date documentation. `cx`, `Scope`, and `use_state` are gone.

**IMPORTANT**: This project uses Dioxus 0.7, which has breaking API changes from previous versions

# Dioxus Dependency

You can add Dioxus to your `Cargo.toml` like this:

```toml
[dependencies]
dioxus = { version = "0.7.1" }

[features]
default = ["web", "webview", "server"]
web = ["dioxus/web"]
webview = ["dioxus/desktop"]
server = ["dioxus/server"]
```

# Launching your application

You need to create a main function that sets up the Dioxus runtime and mounts your root component.

```rust
use dioxus::prelude::*;

fn main() {
	dioxus::launch(App);
}

#[component]
fn App() -> Element {
	rsx! { "Hello, Dioxus!" }
}
```

Then serve with `dx serve`:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
dx serve
```

# UI with RSX

```rust
rsx! {
	div {
		class: "container", // Attribute
		color: "red", // Inline styles
		width: if condition { "100%" }, // Conditional attributes
		"Hello, Dioxus!"
	}
	// Prefer loops over iterators
	for i in 0..5 {
		div { "{i}" } // use elements or components directly in loops
	}
	if condition {
		div { "Condition is true!" } // use elements or components directly in conditionals
	}

	{children} // Expressions are wrapped in brace
	{(0..5).map(|i| rsx! { span { "Item {i}" } })} // Iterators must be wrapped in braces
}
```

# Assets

The asset macro can be used to link to local files to use in your project. All links start with `/` and are relative to the root of your project.

```rust
rsx! {
	img {
		src: asset!("/assets/image.png"),
		alt: "An image",
	}
}
```

## Styles

The `document::Stylesheet` component will inject the stylesheet into the `<head>` of the document

```rust
rsx! {
	document::Stylesheet {
		href: asset!("/assets/styles.css"),
	}
}
```

# Components

Components are the building blocks of apps

* Component are functions annotated with the `#[component]` macro.
* The function name must start with a capital letter or contain an underscore.
* A component re-renders only under two conditions:
	1.  Its props change (as determined by `PartialEq`).
	2.  An internal reactive state it depends on is updated.

```rust
#[component]
fn Input(mut value: Signal<String>) -> Element {
	rsx! {
		input {
            value,
			oninput: move |e| {
				*value.write() = e.value();
			},
			onkeydown: move |e| {
				if e.key() == Key::Enter {
					value.write().clear();
				}
			},
		}
	}
}
```

Each component accepts function arguments (props)

* Props must be owned values, not references. Use `String` and `Vec<T>` instead of `&str` or `&[T]`.
* Props must implement `PartialEq` and `Clone`.
* To make props reactive and copy, you can wrap the type in `ReadOnlySignal`. Any reactive state like memos and resources that read `ReadOnlySignal` props will automatically re-run when the prop changes.

# State

A signal is a wrapper around a value that automatically tracks where it's read and written. Changing a signal's value causes code that relies on the signal to rerun.

## Local State

The `use_signal` hook creates state that is local to a single component. You can call the signal like a function (e.g. `my_signal()`) to clone the value, or use `.read()` to get a reference. `.write()` gets a mutable reference to the value.

Use `use_memo` to create a memoized value that recalculates when its dependencies change. Memos are useful for expensive calculations that you don't want to repeat unnecessarily.

```rust
#[component]
fn Counter() -> Element {
	let mut count = use_signal(|| 0);
	let mut doubled = use_memo(move || count() * 2); // doubled will re-run when count changes because it reads the signal

	rsx! {
		h1 { "Count: {count}" } // Counter will re-render when count changes because it reads the signal
		h2 { "Doubled: {doubled}" }
		button {
			onclick: move |_| *count.write() += 1, // Writing to the signal rerenders Counter
			"Increment"
		}
		button {
			onclick: move |_| count.with_mut(|count| *count += 1), // use with_mut to mutate the signal
			"Increment with with_mut"
		}
	}
}
```

## Context API

The Context API allows you to share state down the component tree. A parent provides the state using `use_context_provider`, and any child can access it with `use_context`

```rust
#[component]
fn App() -> Element {
	let mut theme = use_signal(|| "light".to_string());
	use_context_provider(|| theme); // Provide a type to children
	rsx! { Child {} }
}

#[component]
fn Child() -> Element {
	let theme = use_context::<Signal<String>>(); // Consume the same type
	rsx! {
		div {
			"Current theme: {theme}"
		}
	}
}
```

# Async

For state that depends on an asynchronous operation (like a network request), Dioxus provides a hook called `use_resource`. This hook manages the lifecycle of the async task and provides the result to your component.

* The `use_resource` hook takes an `async` closure. It re-runs this closure whenever any signals it depends on (reads) are updated
* The `Resource` object returned can be in several states when read:
1. `None` if the resource is still loading
2. `Some(value)` if the resource has successfully loaded

```rust
let mut dog = use_resource(move || async move {
	// api request
});

match dog() {
	Some(dog_info) => rsx! { Dog { dog_info } },
	None => rsx! { "Loading..." },
}
```

# Routing

All possible routes are defined in a single Rust `enum` that derives `Routable`. Each variant represents a route and is annotated with `#[route("/path")]`. Dynamic Segments can capture parts of the URL path as parameters by using `:name` in the route string. These become fields in the enum variant.

The `Router<Route> {}` component is the entry point that manages rendering the correct component for the current URL.

You can use the `#[layout(NavBar)]` to create a layout shared between pages and place an `Outlet<Route> {}` inside your layout component. The child routes will be rendered in the outlet.

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
	#[layout(NavBar)] // This will use NavBar as the layout for all routes
		#[route("/")]
		Home {},
		#[route("/blog/:id")] // Dynamic segment
		BlogPost { id: i32 },
}

#[component]
fn NavBar() -> Element {
	rsx! {
		a { href: "/", "Home" }
		Outlet<Route> {} // Renders Home or BlogPost
	}
}

#[component]
fn App() -> Element {
	rsx! { Router::<Route> {} }
}
```

```toml
dioxus = { version = "0.7.1", features = ["router"] }
```

# Fullstack

Fullstack enables server rendering and ipc calls. It uses Cargo features (`server` and a client feature like `web`) to split the code into a server and client binaries.

```toml
dioxus = { version = "0.7.1", features = ["fullstack"] }
```

## Server Functions

Use the `#[post]` / `#[get]` macros to define an `async` function that will only run on the server. On the server, this macro generates an API endpoint. On the client, it generates a function that makes an HTTP request to that endpoint.

```rust
#[post("/api/double/:path/&query")]
async fn double_server(number: i32, path: String, query: i32) -> Result<i32, ServerFnError> {
	tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	Ok(number * 2)
}
```

## Hydration

Hydration is the process of making a server-rendered HTML page interactive on the client. The server sends the initial HTML, and then the client-side runs, attaches event listeners, and takes control of future rendering.

### Errors
The initial UI rendered by the component on the client must be identical to the UI rendered on the server.

* Use the `use_server_future` hook instead of `use_resource`. It runs the future on the server, serializes the result, and sends it to the client, ensuring the client has the data immediately for its first render.
* Any code that relies on browser-specific APIs (like accessing `localStorage`) must be run *after* hydration. Place this code inside a `use_effect` hook.
