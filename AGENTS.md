# Browser AI Assistant - Developer Guide for Agents

## 1. Project Overview

This project is a browser integration system connecting a Chrome Extension frontend with a local Rust backend.

- **Backend**: Rust (Axum, Tokio, Rig-core)
- **Frontend**: Chrome Extension (Vanilla JS, HTML, CSS)
- **AI Engine**: Google Gemini API via Rig-core in Backend

## 2. Backend (Rust)

### Build & Run

- **Directory**: `backend/`
- **Run Dev**: `cargo run` (Listens on port 3000)
- **Run Release**: `cargo run --release`
- **Build Release**: `cargo build --release`
- **Check**: `cargo check`

### Testing & Linting

- **Run All Tests**: `cargo test`
- **Run Single Test**: `cargo test <test_name> -- --nocapture`
- **Lint**: `cargo clippy` (Ensure no warnings before committing)
- **Format (Root)**: `npm run format:rust` (Formats backend from project root)
- **Format (Local)**: `cargo fmt` (Standard Rust formatting)

### Code Style & Conventions

- **Edition**: Rust 2024
- **Async Runtime**: Tokio
- **Web Framework**: Axum
- **AI Integration**: `rig-core` for building LLM-powered applications and agents.
- **Error Handling**:
  - Use `Result` and the `?` operator. Avoid `unwrap()` or `expect()` in production paths.
  - Use `thiserror` for defining custom error types that map to HTTP responses.
- **Logging**: Use `tracing` macros (`info!`, `warn!`, `error!`). Do not use `println!`.
- **State Management**: Use `Arc<RwLock<AppState>>` or `Arc<AppState>` for shared state.
- **Imports**: Grouped by (1) standard library, (2) external crates, (3) local modules.

  ```rust
  use std::sync::Arc;

  use axum::{extract::State, Json};
  use rig::completion::Prompt;

  use crate::error::AppError;
  ```

- **Serialization**: Use `serde` with `#[derive(Serialize, Deserialize)]`.
- **Typing**: Use strong typing; avoid `serde_json::Value` where a struct can be defined.

### Module Structure

- `agent`: Core agent logic, behavioral definitions, and prompt templates. Uses Rig-core client.
- `config`: Environment variable loading (`dotenvy`) and configuration structs for the application.
- `dtos`: Data Transfer Objects for standardized API communication between frontend and backend.
- `error`: Centralized error types and Axum `IntoResponse` implementations for consistent errors.
- `handler`: Request handlers for HTTP routes and WebSocket connections. Implements app logic.
- `llm`: Logic for interfacing with LLM providers (Google Gemini) and Rig client initialization.
- `models`: Core data structures and internal logic models used throughout the backend.
- `routes`: API route definitions, path mapping, and middleware layer configuration (CORS, tracing).
- `state`: Global application state accessible via Axum extractors, shared across all handlers.
- `tools`: Implementations of tools/functions (e.g., search, web navigation) that agents can call.
- `utils`: Shared utilities, helpers for data manipulation, and streaming response logic.

## 3. Frontend (Browser Extension)

### Entry Points

- `sidepanel.js`: Main logic for the Chat UI in the browser's side panel.
- `background.js`: Extension service worker managing lifecycle and events.
- `content.js`: Script injected into pages to extract content and take screenshots.
- `popup.js`: UI logic for the extension's popup menu.
- `offscreen.js`: Handles DOM parsing and heavy tasks in a separate document.

### Testing & Linting

- **Directory**: Project Root
- **Run All Tests**: `npm test` (Runs Jest tests in `tests/extension/`)
- **Run Single Test**: `npm test -- -t "test name"`
- **Format Check**: `npm run format:check` (Prettier check)
- **Format Fix**: `npm run format` (Prettier write)

### Code Style (JavaScript)

- **Standard**: Vanilla JavaScript (ES6+), no build step or bundlers.
- **Naming**: `camelCase` for variables and functions, `PascalCase` for classes.
- **Async**: Use `async/await` over raw Promises for clarity.
- **Error Handling**: Use `try/catch` blocks for all async operations and API calls.
- **DOM**: Use `document.querySelector` and `document.getElementById`.
- **APIs**: Prefer `chrome.storage.local` for state over `localStorage`.

### Code Style (CSS)

- **Theming**: Support Light/Dark modes using CSS variables (`:root` vs `[data-theme="dark"]`).
- **Layout**: Use Flexbox or CSS Grid for all layout tasks.
- **Scrollbars**: Custom styling required for consistent cross-theme appearance.

## 4. Agent Rules & Guidelines

- **No Hallucinations**: Never invent API endpoints or library functions. Verify with search.
- **Test Integrity**: Never delete or bypass tests. Fix the code to make tests pass.
- **LSP Usage**: Use `lsp_diagnostics` to verify code quality before finishing a task.
- **Documentation**: Update `AGENTS.md` when introducing new patterns or modules.
- **Atomic Commits**: Use Conventional Commits (`feat:`, `fix:`, `chore:`) for atomic changes.
- **Pre-Commit**: Always run `npm run format` and `npm run format:rust` before committing.
- **Context Awareness**: Be aware of Chrome Extension script scopes (Sidepanel vs Content).
- **Directory Verification**: Verify parent directories exist using `ls` before creating new files.
- **Command Quoting**: Quote file paths with spaces (e.g., `rm "path with spaces/file.txt"`).

## 5. General Guidelines

### Proactiveness

- Always verify if a fix requires changes in both Backend and Frontend.
- If a new feature requires a dependency, add it to `Cargo.toml` or `extension/lib/`.

### Privacy & Security

- **Secrets**: Never hardcode API keys. Use `.env` file loaded by `dotenvy`.
- **Sanitization**: All text sent to AI must be sanitized in `privacy.rs` (remove PII).

### Release Management

- For distribution: Copy `extension/`, `backend.exe` (release), `README_CEPAT.txt`, `run_backend.bat`.
- Package as a ZIP with version prefix (e.g., `v1.2.0_package.zip`).

## 6. Troubleshooting

- **"Image state not clearing"**: Ensure logic handles both text and image message states.
- **"Connection failed"**: Verify backend is on port 3000 and `GOOGLE_API_KEY` is set.
- **"Extension not updating"**: Click the "Refresh" icon in `chrome://extensions`.
- **"CORS Errors"**: Check Axum `tower_http::cors` configuration in `routes.rs`.
- **"LSP Failures"**: If LSP fails, ensure the correct workspace root is opened.
- **"Rig Client Error"**: Ensure the `GOOGLE_API_KEY` is valid and has sufficient quota.
- **"404 Not Found"**: Verify that the route is defined in `routes.rs` and matches the path.

## 7. AI Integration Best Practices

- **Contextual Prompts**: Always include page context (URL, Title, Body) in system prompts.
- **Token Management**: Be mindful of large screenshots; downscale if possible before sending.
- **Streaming**: Use Server-Sent Events (SSE) or WebSockets for real-time AI responses.
- **Tool Selection**: Only provide tools relevant to the current agent's behavior.
- **Fallback Logic**: Implement fallbacks for API timeouts or rate limits.
