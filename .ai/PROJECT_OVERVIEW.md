# Terminal-Session-Proxy-Manager (proxy-cli-rs)
**Deep AI Context & Architecture Overview**

## 1. Project Purpose
A universal, highly configurable Command Line Interface (CLI) toolkit written in Rust for managing proxy connections (SOCKS5, HTTP) within terminal environments.
It acts as a centralized proxy manager, allowing users to switch profiles, benchmark latencies, auto-select the fastest proxy, and dynamically generate export scripts for `curl`, `docker`, `git`, and shell environments.

## 2. Technology Stack & Core Dependencies
- **Language:** Rust 1.70+
- **CLI Parsing:** `clap` (v4) with `derive` features. Handles deep subcommand trees.
- **Asynchronous Runtime:** `tokio` (multi-threaded executor).
- **Networking:** `reqwest` (with `socks` feature) for HTTP requests and latency probing, `tokio-socks` for raw socket connection checks.
- **Terminal User Interface (TUI):** `ratatui` + `crossterm` for the interactive dashboard (`src/cmd/dash.rs`).
- **Interactive Prompts:** `dialoguer` for terminal menus (e.g., interactive profile switching).
- **Serialization:** `serde` + `serde_json` for configuration management (`config.json`, `settings.json`).
- **File System:** `dirs` crate to locate OS-specific config directories (e.g., `~/.config/terminal-session-proxy-manager`).
- **Error Handling:** `anyhow` for application-level errors, `thiserror` for library-level error definitions.

## 3. Detailed Architecture & Codebase Structure
The project is strictly separated into CLI definition, business logic commands, and configuration state management.

### `src/` (Core Logic)
- **`main.rs`**: Application entry point. Initializes the `cli` parser, loads configurations, and routes subcommands to their respective handlers in `src/cmd/`.
- **`cli.rs`**: Defines the `clap` CLI structures. **CRITICAL:** All `///` doc comments here are inherently bilingual (formatted as `English | Russian`) to provide dual-language `--help` output without complex runtime generation.

### `src/config/` (State Management)
- **`app.rs`**: Manages the main `config.json`. Contains proxy profiles, ping targets, and geo-location APIs. It handles the dynamic generation of default profiles (`default` and `custom`) if no config exists.
- **`settings.rs`**: Manages `settings.json`, handling global preferences like the UI language (`ru` or `en`) and custom config paths.
- **`i18n.rs`**: Internationalization engine. Loads strings from `locales/*.json` based on the active language.

### `src/cmd/` (Command Implementations)
- **`dash.rs`**: The interactive TUI dashboard. Uses a raw terminal mode event loop to render proxy statuses, real-time pings, and IP geolocation.
- **`best.rs` / `benchmark.rs`**: Concurrently spawns `tokio` tasks to ping all available proxy profiles against high-availability targets (e.g., Google, Cloudflare) and sorts them by latency.
- **`env.rs` / `git.rs`**: Generates terminal-specific commands (like `export HTTP_PROXY=...` or `git config --global http.proxy ...`).
- **`import_cmd.rs`**: Parses and imports proxy configurations from local files or remote subscription URLs.
- **`diagnose.rs`**: Performs deep network diagnostics, checking if local sockets are bound and if external endpoints are reachable through the active proxy.

### `locales/` (Translations)
- `en.json` and `ru.json`: JSON key-value stores containing all user-facing text (excluding CLI help docs). Must always be kept strictly in sync.

## 4. Design Philosophy & Constraints
- **Zero Hardcoded Private Data:** The default configuration must NEVER include personal, private, or obscure proxy profiles (e.g., specific user VPS IPs or custom ports like `2080`). It must always fallback to generic examples (`127.0.0.1:1080`).
- **Stateless Execution:** The CLI runs ephemerally. To alter the user's current shell, commands like `proxy-cli env on` generate `export` statements intended to be evaluated by the shell (e.g., `eval $(proxy-cli env on)`).
- **Graceful Degradation:** If the active proxy fails, network checks (like geolocation) should handle timeouts gracefully without crashing the application.
