# ThrallUI

Dioxus fullstack workspace with terminal interface for Claude CLI.

## Packages

- **api**: Backend WebSocket server and session management
- **ui**: Shared UI components
- **web**: Web application
- **desktop**: Desktop application (template)
- **mobile**: Mobile application (template)

## Terminal App Setup

### Prerequisites

- Rust 1.75+
- Claude CLI installed
- Dioxus CLI: `cargo install dioxus-cli`

### Configuration

Create a `.env` file:

```bash
export RUST_LOG="info"
export THRALLUI_ALLOWED_DIRS="/home/user/projects"
export THRALLUI_CLAUDE_PATH="claude"
export THRALLUI_MAX_SESSIONS="10"
```

**Environment Variables**:

- `THRALLUI_ALLOWED_DIRS`: Base directory whose subdirectories will be available for terminal
  sessions
- `THRALLUI_CLAUDE_PATH`: Path to the Claude CLI executable (default: "claude")
- `THRALLUI_MAX_SESSIONS`: Maximum number of concurrent terminal sessions (default: 10)

**Log Levels**: Set `RUST_LOG` to control verbosity:

- `error`: Only errors
- `warn`: Errors and warnings
- `info`: General information (default)
- `debug`: Detailed debugging info
- `trace`: Very verbose tracing

### Running

```bash
# Development with hot reload
cd packages/web
dx serve --features server,web --hot-reload

# Production
cargo build --release -p web --features server
./target/release/web
```

### Usage

1. Navigate to [http://localhost:8080/terminal](http://localhost:8080/terminal)
2. Create a new session with name and directory
3. Select the session from the list
4. Interact with Claude CLI in real-time

## Architecture

### Backend (packages/api)

- WebSocket server using Dioxus fullstack
- PTY-based process management
- Session lifecycle management
- Directory whitelisting for security

### UI (packages/ui)

- SessionManager: Create new sessions
- SessionList: View active sessions
- TerminalView: Terminal output and input

### Web (packages/web)

- WebSocket client using `use_websocket`
- Terminal route
- Real-time communication

## Security

- Only directories in `THRALLUI_ALLOWED_DIRS` can be accessed
- Paths are canonicalized to prevent traversal
- Session limits prevent resource exhaustion

## Development

```bash
# Check all packages
cargo check --workspace --all-features

# Test
cargo test --workspace --all-features

# Format
cargo fmt --all

# Lint
cargo clippy --workspace --all-features
```

## Troubleshooting

### "Directory not allowed"

Ensure directory is in `THRALLUI_ALLOWED_DIRS` and exists.

### "Claude CLI not found"

Set `THRALLUI_CLAUDE_PATH` to full path or ensure `claude` is in PATH.

### WebSocket connection failed

Check server is running and firewall settings.
