//! This crate contains all shared UI for the workspace.

mod hero;
pub use hero::Hero;

mod navbar;
pub use navbar::Navbar;

mod echo;
pub use echo::Echo;

// New: Terminal components
pub mod terminal;

pub use terminal::{SessionList, SessionManager, TerminalView};
