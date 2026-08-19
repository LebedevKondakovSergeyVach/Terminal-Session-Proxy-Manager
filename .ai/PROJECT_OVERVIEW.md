# Terminal Session Proxy Manager — Architecture Overview

Reference material for AI agents and new contributors. The binding rules are in
[`AGENTS.md`](../AGENTS.md); this document explains the shape of the codebase and
why it is arranged the way it is.

## 1. Purpose

A single-binary Rust CLI for managing proxy connections (SOCKS5, HTTP) inside a
terminal session. It stores named proxy profiles, switches between them,
benchmarks their latency, picks the fastest one, diagnoses connectivity, and
generates the configuration that `curl`, Docker, Git, JVM tooling and the shell
itself need.

## 2. Technology stack

| Concern | Crate | Notes |
| :--- | :--- | :--- |
| Language | Rust, edition 2024 | MSRV `1.85`, declared as `rust-version` in `Cargo.toml` and checked in CI |
| CLI parsing | `clap` 4 (derive, `env`, `wrap_help`) | Deep subcommand tree in `src/cli.rs` |
| Async runtime | `tokio` | Only `macros`, `rt-multi-thread`, `time` — not `full` |
| HTTP | `reqwest` (`json`, `socks`) | Latency probes, geolocation, throughput |
| TUI | `ratatui` + `crossterm` | The `dash` dashboard |
| Prompts | `dialoguer` | Interactive profile selection |
| Progress | `indicatif` | Spinners and the speedtest bar |
| Serialization | `serde` + `serde_json` | Both config files |
| Paths | `dirs` | OS config directory |
| Errors | `anyhow` + `thiserror` | Application boundary vs typed variants |

There is no `tokio-socks` dependency; SOCKS support comes from the `socks`
feature of `reqwest`. Raw port checks use `std::net::TcpStream::connect_timeout`.

## 3. Codebase structure

### `src/` — entry point and cross-cutting modules

- **`main.rs`** — Resolves config paths and language from raw argv
  (`preparse`), builds a localised clap command, parses, and dispatches. Returns
  `ExitCode`. Deliberately contains no business logic.
- **`lib.rs`** — Library root. Everything worth testing lives beneath it, which
  is why `main.rs` stays thin.
- **`cli.rs`** — The clap command tree only. Doc comments here are bilingual
  (`English | Русский`) and act as the static fallback; `main.rs` overrides
  subcommand descriptions at runtime from the `cmd_*` locale keys.
- **`proxy_env.rs`** — The single definition of which environment variables this
  tool manages and how they are rendered and quoted for a shell. `env`, `run`
  and `dash` all go through it.
- **`shell_handoff.rs`** — Owns the `$HOME` marker files. A child process cannot
  modify its parent's environment, so the dashboard writes `export` lines to
  `~/.terminal-session-proxy-manager-eval` and the shell function installed by
  `init` evaluates and deletes that file on exit.
- **`error.rs`** — `ProxyError`. Only errors a caller might match on earn a
  variant; anything merely displayed stays an `anyhow` error.

### `src/config/` — state

- **`app.rs`** — `config.json`: profiles, ping targets, diagnostic endpoints,
  geolocation APIs, and the IPv4/IPv6/health-check/speedtest URLs. Owns path
  resolution and the load/save cycle.
- **`settings.rs`** — `settings.json`: interface language and an optional custom
  config path.
- **`profile.rs`** — The `Profile` struct and `validate`, which allowlists the
  characters a host may contain and checks port and protocol.
- **`i18n.rs`** — Loads `locales/*.json`, embedded with `include_str!`. `t` looks
  up a key; `format` substitutes `{}` placeholders in order.

### `src/cmd/` — one module per subcommand

- **`dash.rs`** — The TUI. An RAII `TerminalGuard` plus a panic hook guarantee
  the terminal is restored on every exit path. Long actions that need a normal
  terminal (an editor, a prompt) are returned as a `DashAction` and performed
  after teardown, not inside the alternate screen.
- **`profile.rs`** — Listing, selecting, adding, removing, and the concurrent
  benchmark. `benchmark_profiles` returns `BenchmarkResult`, whose `avg_ms` is
  an `Option` so an unreachable proxy is distinguishable from a slow one.
  `best` and `benchmark` live here — there is no `best.rs` or `benchmark.rs`.
- **`git_cmd.rs`** — Global Git proxy configuration. Named `git_cmd` to avoid
  colliding with the `git` subcommand name.
- **`env.rs`** — Emits `export` / `unset` statements for `eval`.
- **`import_cmd.rs`** — Parses a full config, a profile map, a profile array, or
  a plain list of proxy URLs, from a file or a URL.
- **`status.rs`**, **`ping.rs`**, **`diagnose.rs`**, **`monitor.rs`**,
  **`speedtest.rs`** — The network-facing reports.
- **`init.rs`**, **`completions.rs`** — Shell integration and completions.
- **`settings.rs`**, **`export_cmd.rs`** — Settings management and export
  formats.

### `locales/`

`en.json` and `ru.json`. Key sets must be identical and no value may be empty;
tests enforce both.

## 4. Design constraints

**No private data in defaults.** `AppConfig::default` may reference only
loopback addresses and well-known public endpoints. A personal VPS address or an
unusual port committed here would ship to every user and silently point them at
someone else's proxy. A test asserts every default profile is `127.0.0.1`.

**Stateless execution.** The CLI is ephemeral. It cannot mutate the calling
shell, so `env on` prints statements intended for `eval`, and the dashboard uses
the hand-off file described above.

**Never destroy user configuration.** Defaults are written only when a config
file is absent. A file that exists but fails to parse is reported and left
untouched.

**Everything a shell evaluates is quoted.** Profile fields originate in a JSON
file that can be edited by hand or imported from a URL, and the output is passed
to `eval`. `shell_quote` at the boundary and `Profile::validate` at the point of
entry are two independent layers of the same defence.

**Graceful degradation.** Network checks time out rather than hang, and a failed
probe is reported as a failure rather than crashing the command.

**Exit codes are interface.** Failure means a non-zero exit, because these
commands are used inside shell conditionals.

## 5. Configuration precedence

- `config.json` — `--config-file` → `TSPM_CONFIG` → `config_path` from
  `settings.json` → OS config directory.
- `settings.json` — `--settings-file` → `TSPM_SETTINGS` → OS config directory →
  `./settings.json` in the working directory.
- Language — `--lang` → `TSPM_LANG` → `lang` from `settings.json` → `ru`.

The working-directory entry is last on purpose: `settings.json` is a common
filename, and letting an arbitrary directory outrank the user's own
configuration would make the tool behave differently depending on where it was
invoked from.
