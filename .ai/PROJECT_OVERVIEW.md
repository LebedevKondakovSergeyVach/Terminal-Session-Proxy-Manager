# Project map

Ownership, execution flows and change entry points for Terminal Session Proxy
Manager. The binding rules are in [`AGENTS.md`](../AGENTS.md). This is a map,
not a file inventory: verify the relevant flow in source before changing it.
Update this document in the same change when a named owner or flow changes.
Dependency versions and feature flags come from `Cargo.toml` and the lockfiles.

## Boundaries and ownership

The binary manages saved proxy profiles and emits configuration for consumers.
A child process cannot modify its parent shell, so selecting a profile and
applying its environment are separate operations.

| Owner | Responsibility |
| :--- | :--- |
| `src/main.rs` | Global flag preparse, localized clap tree, dispatch; inline `run`, `prompt`, `debug`, `config` |
| `src/lib.rs`, `src/cmd/mod.rs`, `src/config/mod.rs` | Public module boundaries |
| `src/cli.rs` | clap command tree and bilingual help fallback |
| `src/config/app.rs` | Profile configuration, defaults, path resolution, load/save guard |
| `src/config/settings.rs` | Language/config-path settings, path resolution, load/save guard |
| `src/config/profile.rs` | Profile data and validation |
| `src/config/i18n.rs`, `locales/` | Embedded translations and placeholder substitution |
| `src/proxy_env.rs` | Managed variables, values and shell quoting |
| `src/shell_handoff.rs` | Home-directory handoff/debug marker files |
| `src/error.rs` | Typed `ProxyError` variants |
| `src/cmd/profile.rs` | Profile operations, benchmark and fastest-profile selection |
| `src/cmd/dash.rs` | TUI, terminal guard and post-teardown actions |
| `src/cmd/env.rs`, `src/cmd/init.rs` | Export/unset output and shell wrapper with embedded completions |
| `src/cmd/completions.rs` | Standalone `completions` subcommand |
| `src/cmd/import_cmd.rs`, `src/cmd/export_cmd.rs` | Import formats and exported configuration |
| `src/cmd/settings.rs`, `src/cmd/git_cmd.rs` | Settings/language and global Git proxy settings |
| `src/cmd/status.rs`, `src/cmd/ping.rs`, `src/cmd/diagnose.rs`, `src/cmd/monitor.rs`, `src/cmd/speedtest.rs` | Network reports and monitoring |
| `tests/cli.rs` | Tests spawning the binary with isolated configuration |
| `shell/`, `configs/` | Shipped shell functions and default configuration |
| `website/` | Generated bilingual documentation site; own contract |

There is no separate `best` or `benchmark` module. `lang` shares the settings
handler. `main.rs` has binary-local tests in addition to library and CLI tests.
SOCKS support comes from reqwest's `socks` feature; raw port checks use
`std::net::TcpStream::connect_timeout`.

## Execution flows

### Startup and configuration

`main::run` → `preparse` path flags → `AppSettings::load` → choose language →
`localised_command` → clap parsing → `AppConfig::load` → `dispatch`.

Preparse stops at `run` or `--`, so child arguments do not retarget the manager.
Global path and language flags are needed before parsing localized help.

| Setting | Precedence |
| :--- | :--- |
| Profile config | `--config-file` → `TSPM_CONFIG` → settings `config_path` → OS config directory |
| Settings | `--settings-file` → `TSPM_SETTINGS` → existing OS settings → existing working-directory settings → OS default path |
| Language | `--lang` → `TSPM_LANG` → settings `lang` → `ru` |

A relative settings `config_path` resolves against the settings file's parent,
not the command's working directory. The OS settings file outranks an existing
`./settings.json`.

For both configuration types: `load_from` distinguishes absence from failure.
Absence may write defaults; failure reports the error, returns defaults marked
`load_failed`, and `save` refuses to overwrite the original file.

### Saved selection and parent-shell environment

Profile mutations → validate new/imported data → `config.save()`.
Saving selection alone does not modify the shell environment.

`proxy on` shell wrapper → binary `env on` → `proxy_env::export_statements` →
`shell_quote` → shell `eval`. `env off` emits unsets from `MANAGED_ENV_VARS`.

The wrapper's `use`/`switch`/`best` branches save selection, preserve command
status, and reapply exports only after success when `ALL_PROXY` is nonempty.
Read `src/cmd/init.rs` for wrapper behavior; direct binary commands cannot
change their parent environment.

### Child command

`dispatch` → `run_through_proxy` → `proxy_env::proxy_env_vars` →
`std::process::Command::envs` → child status → `ExitCode`.

The environment applies to that child. The parent shell stays unchanged;
signal termination is represented as failure because `ExitCode` cannot carry
a signal.

### Dashboard handoff

`dash::run_dashboard` → terminal guard/event loop → selected `DashAction` →
terminal teardown → `handle_action`.

Enter saves selection and returns `Apply`; that action writes quoted exports
through `shell_handoff::write_exports`. The shell wrapper evaluates and removes
the handoff file after the binary returns. Space saves selection without that
handoff. Editors and prompts run only after terminal teardown.

### Documentation generation

Canonical Markdown (`README*.md`, `docs/`, contributing and changelogs) →
`website/scripts/docs-manifest.mjs` → `website/scripts/sync-docs.mjs` →
cleanup/directives/link rewriting/frontmatter → ignored content → Astro build
and link validation.

The two home and two 404 pages are hand-authored. `<!--site:…-->` comments add
site components while sources remain readable CommonMark on GitHub. Read
[`website/AGENTS.md`](../website/AGENTS.md) before changing this pipeline.

## Where to start a change

| Task | Entry points and related updates |
| :--- | :--- |
| Command or flag | `src/cli.rs`, dispatch/handler, both locales, `tests/cli.rs`, bilingual user docs/changelogs |
| Config path or malformed-file behavior | `src/main.rs`, `src/config/app.rs`, `src/config/settings.rs`, isolated CLI tests |
| Proxy values or quoting | `src/proxy_env.rs`, `src/config/profile.rs`, env/run/dashboard callers |
| Profile import or fastest selection | `src/cmd/import_cmd.rs`, `src/cmd/profile.rs`, validation and failure status |
| Shell integration | `src/cmd/init.rs`, `src/shell_handoff.rs`, both shipped functions in `shell/` |
| TUI action | `src/cmd/dash.rs`, terminal cleanup, translations, handoff and screenshot impact |
| Site content, theme or generator | Canonical Markdown, manifest, `website/astro.config.mjs`, website rules/skill |
| Dependency, CI or release | Cargo/npm manifest and lockfile, workflow source, CI rule and release/security skills |
| Agent instructions | Root/area contract, matching rule/pointer/skill, instruction audit |

Use `tests/cli.rs`'s `Cli` helper for binary tests: it isolates config, settings,
language and color. Existing network-facing commands are not exercised there;
test their pure logic without external network access.
