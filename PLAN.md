# ThrallUI - Terminal Interaction App Plan

## Architecture Overview

### Frontend (Dioxus Web)
- Split-screen layout with session manager (left) and terminal view (right)
- WebSocket client for real-time communication
- Component-based architecture

### Backend (Dioxus Server/Axum)
- WebSocket server for streaming terminal output
- Process manager for Claude CLI instances
- Session lifecycle management
- Configuration via environment variables

---

## Step 1: Project Structure and Dependencies

### Task 1.1: Update project dependencies and structure
**File**: `Cargo.toml` (modify existing)
**Description**: Add required dependencies for WebSocket, async runtime, terminal handling, and process management.

Dependencies to add:
```toml
[dependencies]
dioxus = { version = "0.6", features = ["web", "router"] }
dioxus-fullstack = "0.6"
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Task 1.2: Create directory structure
**Files**: Create directory structure
```
packages/thrallui/src/
├── main.rs
├── components/
│   ├── mod.rs
│   ├── layout.rs
│   ├── session_manager.rs
│   ├── session_list.rs
│   └── terminal_view.rs
├── models/
│   ├── mod.rs
│   ├── session.rs
│   └── message.rs
├── server/
│   ├── mod.rs
│   ├── websocket.rs
│   ├── process_manager.rs
│   └── session_handler.rs
├── state/
│   ├── mod.rs
│   └── app_state.rs
└── config/
    ├── mod.rs
    └── environment.rs
```

---

## Step 2: Core Data Models and Types

### Task 2.1: Define message types
**File**: `packages/thrallui/src/models/message.rs`
**Description**: Define message types for WebSocket communication between client and server.

Structure:
- `ClientMessage`: Messages sent from client to server
  - `CreateSession { name: String, directory: String }`
  - `SendInput { session_id: String, input: String }`
  - `SelectSession { session_id: String }`
  - `CloseSession { session_id: String }`
- `ServerMessage`: Messages sent from server to client
  - `SessionCreated { session_id: String, name: String }`
  - `SessionList { sessions: Vec<SessionInfo> }`
  - `TerminalOutput { session_id: String, data: String }`
  - `SessionClosed { session_id: String }`
  - `Error { message: String }`

### Task 2.2: Define session model
**File**: `packages/thrallui/src/models/session.rs`
**Description**: Define session data structures.

Structure:
- `Session`: Full session state on server
  - `id: String`
  - `name: String`
  - `directory: String`
  - `created_at: DateTime`
  - `process: Option<Child>`
  - `output_buffer: Vec<String>`
- `SessionInfo`: Lightweight session info for client
  - `id: String`
  - `name: String`
  - `created_at: String`
- `SessionConfig`: Configuration for creating sessions
  - `name: String`
  - `directory: String`

### Task 2.3: Create models module
**File**: `packages/thrallui/src/models/mod.rs`
**Description**: Export models module.

---

## Step 3: Configuration and Environment

### Task 3.1: Environment configuration
**File**: `packages/thrallui/src/config/environment.rs`
**Description**: Load and validate environment variables.

Configuration:
- `THRALLUI_ALLOWED_DIRS`: Comma-separated list of allowed base directories
- `THRALLUI_CLAUDE_PATH`: Path to Claude CLI executable (default: "claude")
- `THRALLUI_WS_PORT`: WebSocket server port (default: 8080)
- `THRALLUI_MAX_SESSIONS`: Maximum concurrent sessions (default: 10)

### Task 3.2: Create config module
**File**: `packages/thrallui/src/config/mod.rs`
**Description**: Export config module.

---

## Step 4: Backend - Process Management

### Task 4.1: Process manager for Claude CLI
**File**: `packages/thrallui/src/server/process_manager.rs`
**Description**: Manage Claude CLI processes with PTY (pseudo-terminal).

Functionality:
- `spawn_claude_process(directory: String) -> Result<Child>`
  - Spawn Claude CLI in specified directory
  - Set up PTY for proper terminal interaction
  - Capture stdout/stderr
- `write_to_process(process: &mut Child, input: &str) -> Result<()>`
  - Write user input to Claude's stdin
- `read_from_process(process: &mut Child) -> Result<String>`
  - Read output from Claude (non-blocking)
  - Handle ANSI codes and formatting

### Task 4.2: Session handler
**File**: `packages/thrallui/src/server/session_handler.rs`
**Description**: Manage session lifecycle and state.

Functionality:
- `SessionManager` struct:
  - `sessions: HashMap<String, Session>`
  - `create_session(config: SessionConfig) -> Result<String>`
  - `get_session(&self, id: &str) -> Option<&Session>`
  - `close_session(&mut self, id: &str) -> Result<()>`
  - `list_sessions(&self) -> Vec<SessionInfo>`
- Implement cleanup on session close
- Handle session limits

### Task 4.3: WebSocket handler
**File**: `packages/thrallui/src/server/websocket.rs`
**Description**: WebSocket server implementation with message routing.

Functionality:
- `handle_websocket(socket: WebSocket, state: AppState)`
  - Handle WebSocket connection lifecycle
  - Parse incoming ClientMessages
  - Route to appropriate handlers
  - Send ServerMessages back to client
- `broadcast_to_session(session_id: &str, message: ServerMessage)`
  - Send updates to all clients watching a session
- Background task to stream Claude output to connected clients

### Task 4.4: Create server module
**File**: `packages/thrallui/src/server/mod.rs`
**Description**: Export server module.

---

## Step 5: Application State

### Task 5.1: Global application state
**File**: `packages/thrallui/src/state/app_state.rs`
**Description**: Shared state accessible from both frontend and backend.

Structure:
- `AppState`:
  - `sessions: Arc<Mutex<SessionManager>>`
  - `config: Config`
  - `active_connections: Arc<Mutex<HashMap<String, Vec<WebSocketSender>>>>`

### Task 5.2: Create state module
**File**: `packages/thrallui/src/state/mod.rs`
**Description**: Export state module.

---

## Step 6: Frontend Components (Parallel Tasks)

### Task 6.1: Main layout component
**File**: `packages/thrallui/src/components/layout.rs`
**Description**: Root component with split-screen layout.

Structure:
- Two-column grid layout (30% left, 70% right)
- CSS flexbox for responsive design
- Render SessionManager (left) and TerminalView (right)

### Task 6.2: Session manager component (left panel)
**File**: `packages/thrallui/src/components/session_manager.rs`
**Description**: Session creation form and controls.

Features:
- Text input for session name
- Directory selector (input or dropdown of allowed dirs)
- "Create Session" button
- Form validation
- Send CreateSession message via WebSocket

### Task 6.3: Session list component (left panel)
**File**: `packages/thrallui/src/components/session_list.rs`
**Description**: Display and select active sessions.

Features:
- List all sessions from state
- Click to select/activate session
- Highlight active session
- Show session name and creation time
- Close button for each session

### Task 6.4: Terminal view component (right panel)
**File**: `packages/thrallui/src/components/terminal_view.rs`
**Description**: Display terminal output and input field.

Features:
- Scrollable terminal output area
  - Display messages with proper formatting
  - Auto-scroll to bottom on new messages
  - ANSI code rendering (optional: use a library)
- Input field at bottom
  - Text input for user messages
  - "Send" button
  - Enter key to send
  - Send SendInput message via WebSocket

### Task 6.5: Create components module
**File**: `packages/thrallui/src/components/mod.rs`
**Description**: Export all components.

---

## Step 7: Frontend State and WebSocket Client

### Task 7.1: WebSocket client integration
**File**: Update `packages/thrallui/src/components/layout.rs`
**Description**: Connect to WebSocket server and manage client state.

Functionality:
- Establish WebSocket connection on component mount
- Listen for ServerMessages:
  - Update session list on SessionCreated/SessionClosed
  - Append terminal output on TerminalOutput
  - Handle errors
- Send ClientMessages for user actions
- Use `use_signal` or `use_resource` for reactive state

### Task 7.2: Client-side state management
**File**: `packages/thrallui/src/state/app_state.rs` (extend)
**Description**: Client-side reactive state.

State:
- `sessions: Signal<Vec<SessionInfo>>`
- `active_session_id: Signal<Option<String>>`
- `terminal_output: Signal<HashMap<String, Vec<String>>>`
- `connection_status: Signal<ConnectionStatus>`

---

## Step 8: Main Application Entry Point

### Task 8.1: Server setup (Axum)
**File**: `packages/thrallui/src/main.rs`
**Description**: Configure Axum server with WebSocket route and static file serving.

Setup:
- Initialize tracing/logging
- Load configuration from environment
- Create AppState
- Set up Axum routes:
  - `/ws` → WebSocket handler
  - `/` → Dioxus app (static files or SSR)
- Start server on configured port

### Task 8.2: Dioxus app initialization
**File**: `packages/thrallui/src/main.rs` (extend)
**Description**: Initialize Dioxus app with router and main component.

Setup:
- Configure Dioxus for web platform
- Set root component to Layout
- Handle hot reload for development

---

## Step 9: Styling and Polish

### Task 9.1: CSS styling
**File**: `packages/thrallui/assets/main.css` (create)
**Description**: Style the application.

Styles:
- Split-screen layout (grid or flexbox)
- Session list styling (hover, active states)
- Terminal view (monospace font, dark theme)
- Input fields and buttons
- Responsive design considerations

### Task 9.2: Asset configuration
**File**: `packages/thrallui/Dioxus.toml` (modify if exists)
**Description**: Configure asset handling and build settings.

---

## Step 10: Testing and Documentation

### Task 10.1: Create README
**File**: `packages/thrallui/README.md` (update existing)
**Description**: Document setup, configuration, and usage.

### Task 10.2: Environment template
**File**: `packages/thrallui/.env.example`
**Description**: Example environment configuration.

```
THRALLUI_ALLOWED_DIRS=/home/user/projects,/home/user/workspace
THRALLUI_CLAUDE_PATH=claude
THRALLUI_WS_PORT=8080
THRALLUI_MAX_SESSIONS=10
```

---

## Implementation Order

1. **Step 1**: Project structure (sequential - foundational)
2. **Step 2 + 3**: Models and config (parallel - independent)
3. **Step 4**: Backend implementation (sequential within step, depends on Step 2)
4. **Step 5**: State management (depends on Steps 2, 3, 4)
5. **Step 6**: Frontend components (parallel tasks 6.1-6.5)
6. **Step 7**: Frontend integration (depends on Step 6)
7. **Step 8**: Main entry point (depends on all previous)
8. **Step 9**: Styling (can be done in parallel with Steps 6-7)
9. **Step 10**: Documentation (final)

---

## Key Technical Considerations

1. **PTY vs Pipes**: Use PTY (portable-pty crate) for proper terminal emulation with Claude CLI
2. **WebSocket vs SSE**: WebSocket for bidirectional communication
3. **Process Cleanup**: Ensure Claude processes are killed when sessions close
4. **Security**: Validate directory paths against allowed list
5. **Error Handling**: Graceful degradation and user-friendly error messages
6. **State Sync**: Keep client and server session lists in sync
7. **Auto-reconnect**: Handle WebSocket disconnections gracefully

---

## Future Iterations (Out of Scope for Initial Implementation)

- Desktop and mobile platform support
- Session persistence (save/restore sessions)
- Multiple terminal types (bash, python REPL, etc.)
- File upload/download integration
- Collaborative sessions (multiple users)
- Terminal themes and customization
- Search within terminal output
- Copy/paste functionality
