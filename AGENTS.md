# Browser AI Assistant - Developer Guide for Agents

## 1. Project Overview
This project is a browser integration system connecting a Chrome Extension frontend with a local Rust backend.
- **Backend**: Rust (Axum, Tokio)
- **Frontend**: Chrome Extension (Vanilla JS, HTML, CSS)
- **AI Engine**: Google Gemini API via Backend

## 2. Backend (Rust)

### Build & Run
- **Directory**: `backend/`
- **Run Dev**: `cargo run` (Listens on port 3000)
- **Run Release**: `cargo run --release`
- **Build Release**: `cargo build --release`
- **Check**: `cargo check`

### Testing & Linting
- **Run All Tests**: `cargo test`
- **Run Single Test**: `cargo test test_name -- --nocapture`
- **Lint**: `cargo clippy` (Ensure no warnings before committing)
- **Format**: `cargo fmt` (Standard Rust formatting)

### Code Style & Conventions
- **Edition**: Rust 2024
- **Async Runtime**: Tokio
- **Web Framework**: Axum
- **Error Handling**:
  - Avoid `unwrap()` or `expect()` in production paths. Use `match` or `?` operator.
  - Use `tracing::info!`, `tracing::warn!`, `tracing::error!` for logging. Do not use `println!`.
- **State Management**: Use `Arc<RwLock<AppState>>` for shared state across threads.
- **Imports**: Group imports by crate, then std.
  ```rust
  use axum::{...};
  use std::sync::Arc;
  ```
- **Serialization**: Use `serde` with `#[derive(Serialize, Deserialize)]`.

### Key Files
- `src/main.rs`: Entry point, server configuration, route definitions.
- `src/ai.rs`: Google Gemini API integration logic.
- `src/ws.rs`: WebSocket handlers for real-time communication.
- `src/privacy.rs`: Data sanitization logic.

## 3. Frontend (Browser Extension)

### Structure
- **Directory**: `extension/`
- **Manifest**: `manifest.json` (Manifest V3)
- **Entry Points**:
  - `sidepanel.html` & `sidepanel.js`: Main Chat UI.
  - `background.js`: Service worker.
  - `content.js`: Page content extractor.

### Development Workflow
1. Load unpacked extension in `chrome://extensions`.
2. To reflect changes:
   - **UI Changes** (HTML/CSS): Close and reopen the side panel.
   - **Script Changes** (JS): Click the "Refresh" icon in `chrome://extensions`.
   - **Manifest Changes**: Remove and re-add the extension.

### Code Style (JavaScript)
- **Format**: Vanilla JavaScript (ES6+).
- **Naming**: `camelCase` for variables and functions.
- **Async**: Use `async/await` over raw Promises.
- **DOM**: Use `document.getElementById` and `querySelector`.
- **No Build Step**: The extension runs raw code. Do not introduce TypeScript or bundlers unless explicitly requested.

### Code Style (CSS)
- **Theme**: Support both Light and Dark modes using CSS variables (`:root` vs `[data-theme="dark"]`).
- **Layout**: Use Flexbox for layout.
- **Scrollbars**: Custom styling required for cross-theme consistency.

## 4. General Guidelines for Agents

### 1. Proactiveness
- When fixing bugs, always verify if the fix requires changes in both Backend and Frontend.
- If a new feature requires a dependency, add it to `Cargo.toml` (Backend) or include the library file (Frontend).

### 2. Privacy & Security
- **API Keys**: Never hardcode API keys. Use `.env` file loaded by `dotenvy` in Rust.
- **Sanitization**: All text sent to AI must be sanitized in `privacy.rs` (remove sensitive PII if possible).

### 3. Git Operations
- **Commit Messages**: Use Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`).
- **Files to Ignore**: Ensure `target/`, `node_modules/`, `.env`, and `*.zip` are in `.gitignore`.

### 4. release Management
- When asked to zip the project:
  1. Create a distribution folder.
  2. Copy `extension/` folder.
  3. Copy `backend/target/release/backend.exe`.
  4. Include `README_CEPAT.txt` and `run_backend.bat`.
  5. Zip with version prefix (e.g., `v1.2.0_package.zip`).

## 5. Troubleshooting Common Issues
- **"Image state not clearing"**: Ensure logic handles both text-only and image-only messages.
- **"Connection failed"**: Verify backend is running on port 3000 and `GOOGLE_API_KEY` is set in `.env`.
- **"CORS Errors"**: The backend is configured to allow `Any` origin for local development flexibility.
