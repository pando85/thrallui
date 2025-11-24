use dioxus::prelude::*;

use ui::Navbar;
use views::{Blog, Home, Terminal};

// Import server functions to register them
#[cfg(not(target_family = "wasm"))]
#[allow(unused_imports)]
use api::{
    create_session, delete_session, get_allowed_directories, get_sessions, terminal_websocket,
};

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebNavbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/terminal")]
    Terminal {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TERMINAL_CSS: Asset = asset!("/assets/terminal.css");

fn main() {
    // Initialize tracing
    #[cfg(target_family = "wasm")]
    tracing_wasm::set_as_global_default();

    #[cfg(not(target_family = "wasm"))]
    {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Initialize server-side state
    #[cfg(not(target_family = "wasm"))]
    {
        use api::{AppState, Config};
        use std::sync::Once;

        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let config = Config::from_env();
            AppState::init(config);
            tracing::info!("AppState initialized");
        });
    }

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TERMINAL_CSS }

        Router::<Route> {}
    }
}

/// A web-specific Router around the shared `Navbar` component
/// which allows us to use the web-specific `Route` enum.
#[component]
fn WebNavbar() -> Element {
    rsx! {
        Navbar {
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::Blog { id: 1 }, "Blog" }
            Link { to: Route::Terminal {}, "Terminal" }
        }

        Outlet::<Route> {}
    }
}
