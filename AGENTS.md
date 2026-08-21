# AGENTS.md

Instructions for AI coding agents working in this repository. Written to the
[agents.md](https://agents.md/) convention; human contributors should read
[`CONTRIBUTING.md`](CONTRIBUTING.md), which covers the same ground in prose.

> Deeper background lives in [`.ai/PROJECT_OVERVIEW.md`](.ai/PROJECT_OVERVIEW.md)
> (architecture), [`.ai/GIT_WORKFLOW.md`](.ai/GIT_WORKFLOW.md) (branching and
> releases) and [`.ai/WORKFLOW_GUIDE.md`](.ai/WORKFLOW_GUIDE.md) (verification).
> This file is the contract; those three are the reference material.

## What this project is

A single-binary Rust CLI that manages SOCKS/HTTP proxy profiles for a terminal
session: switching profiles, benchmarking latency, diagnosing connectivity, and
emitting the environment exports a shell evaluates. Targets macOS and Linux with
zsh and bash. Interface is bilingual (English and Russian).

## Commands

```bash
cargo build                                    # debug build
cargo test                                     # all tests: unit + integration
cargo test --lib                               # unit tests only (fast)
cargo test --test cli                          # end-to-end binary tests
cargo fmt --all                                # apply formatting
cargo clippy --all-targets -- -D warnings      # lint, exactly as CI does
cargo run -- <subcommand>                      # run locally
```

**Before you claim a task is done, run all three:**

```bash
cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked
```

CI runs these with `RUSTFLAGS: -D warnings`, plus an MSRV check against the
`rust-version` in `Cargo.toml`. Warnings are build failures — do not leave them.

## Layout

| Path | Contains |
| :--- | :--- |
| `src/main.rs` | Entry point. Pre-parses global flags, localises help, dispatches. Thin by design. |
| `src/lib.rs` | Library root; everything testable lives under it. |
| `src/cli.rs` | The clap command tree, and nothing else. |
| `src/cmd/` | One module per subcommand, each owning its own output. |
| `src/config/` | `config.json` (`app.rs`), `settings.json` (`settings.rs`), profiles (`profile.rs`), translations (`i18n.rs`). |
| `src/proxy_env.rs` | **The** definition of the proxy environment variables. |
| `src/shell_handoff.rs` | The `$HOME` files used to change the parent shell's environment. |
| `src/error.rs` | `ProxyError` — variants callers may want to match on. |
| `locales/` | `en.json` and `ru.json`, embedded at compile time. |
| `tests/cli.rs` | End-to-end tests that spawn the real binary. |

Note there is no `best.rs`, `benchmark.rs`, or `git.rs`: benchmarking and
best-profile selection live in `src/cmd/profile.rs`, and Git integration is
`src/cmd/git_cmd.rs`.

## Rules

### 1. Every user-facing string is translated

Never hard-code display text in `println!` or TUI rendering.

- Use `i18n.t("key")`, or `i18n.format("key", &[a, b])` for `{}` placeholders.
- Add the key to **both** `locales/en.json` and `locales/ru.json` in the same
  change. A test enforces that the two files define identical key sets and that
  no value is empty, so a one-sided addition fails the build.
- In `src/cli.rs`, clap doc comments stay bilingual as
  `/// English text | Русский текст`. These are the static fallback; `main.rs`
  replaces subcommand descriptions at runtime from `cmd_<subcommand>` keys.
- Adding a subcommand therefore requires a `cmd_<name>` key in both locales.
  A test asserts this; without it `--help` prints the raw key.

Error text returned through `ProxyError` is English. Attach the translated
sentence as `anyhow` context so the user sees their language first and the
precise cause second:

```rust
return Err(ProxyError::ProfileNotFound(key.to_string()))
    .context(i18n.format("profile_not_found", &[key]));
```

### 2. Never destroy a user's configuration

`AppConfig::load` and `AppSettings::load` fall back to defaults only when a file
is **absent**. A file that exists but fails to parse is reported on stderr and
left byte-for-byte intact. Preserve this. Writing defaults over an unparsable
config silently discards every profile the user has, which is exactly the bug
that `a_malformed_config_is_reported_and_left_byte_for_byte_intact` guards.

### 3. Anything a shell evaluates must be quoted

`env on` output and the dashboard hand-off file are consumed by
`eval "$(...)"`. Every interpolated value must pass through
`proxy_env::shell_quote`, because profile fields come from a JSON file the user
can edit or import from a URL. `Profile::validate` is the second layer: it
allowlists the characters a host may contain and runs before any profile is
persisted, from `profile set` or from an import.

### 4. There is one definition of the proxy environment

`src/proxy_env.rs`. `env on`, `run`, and the TUI dashboard all call it. They
used to build the list separately and had silently diverged. Do not reintroduce
a local copy; add to `MANAGED_ENV_VARS` instead, which keeps `env off`
symmetrical automatically.

### 5. Exit codes are part of the interface

A command that fails must return `Err`, not print an error and return `Ok`.
People write `proxy profile use "$p" || fallback`. Handlers return
`anyhow::Result`; `main` maps that to a non-zero `ExitCode`. `run` propagates
its child's exit code verbatim.

### 6. Defaults carry no private data

`AppConfig::default` must only ever reference loopback (`127.0.0.1`) and
well-known public endpoints. Never commit a personal VPS address or an unusual
port. A test asserts every default profile is loopback.

`configs/config.default.json` must match `AppConfig::default()`; a test compares
them. Regenerate with:

```bash
cargo run -- --config-file /dev/null config show > configs/config.default.json
```

### 7. The TUI must always restore the terminal

`src/cmd/dash.rs` uses an RAII `TerminalGuard` plus a panic hook. If either is
removed, a panic inside the draw loop leaves the user on the alternate screen in
raw mode with no visible error. Any new early return from the dashboard must go
through the guard.

## Testing expectations

Tests are behavioural and named as sentences describing the guarantee
(`using_an_unknown_profile_exits_non_zero`), not `test_foo_works`. Each asserts
one thing.

- Pure logic → a `#[cfg(test)] mod tests` beside the code.
- Anything involving argv, exit codes, or files → `tests/cli.rs`.
- Integration tests **must** isolate state via the `TSPM_CONFIG` and
  `TSPM_SETTINGS` environment variables (see the `Cli` helper in
  `tests/cli.rs`). A test that touches the real `~/.config` is a bug.
- Do not add tests that require network access; they are slow and flaky. Cover
  the pure logic instead — `status`, `ping`, `speedtest`, `monitor` and
  `benchmark` are deliberately untested end-to-end.

Fixing a bug means adding the test that fails without the fix.

## Configuration precedence

Know this before changing path resolution:

- `config.json`: `--config-file` → `TSPM_CONFIG` → `config_path` in
  `settings.json` → OS config dir.
- `settings.json`: `--settings-file` → `TSPM_SETTINGS` → OS config dir →
  `./settings.json` in the working directory.
- Language: `--lang` → `TSPM_LANG` → `lang` in `settings.json` → `ru`.

`--config-file`, `--settings-file` and `--lang` are read from raw argv by
`main::preparse` *before* clap parses, because the help text is localised while
the command tree is being built. clap remains the authority afterwards.

## Documentation

A change to commands or configuration is incomplete until these agree:

- `README.md` **and** `README.ru.md` (kept in lockstep)
- the relevant file in `docs/` and its `.ru.md` twin
- `CHANGELOG.md`, under `Unreleased`

If you materially change the TUI, say so in your summary — the screenshot in the
README will need retaking. GitHub caches images aggressively, so a new
screenshot needs a **new filename**, not an overwrite.

## Conventions

- Commits follow Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:`,
  `refactor:`, `test:`, `perf:`, `ci:`, `build:`).
- Comments explain *why*, not *what*. Do not narrate the code.
- `unsafe` is forbidden by `[lints.rust]` in `Cargo.toml`.

## Branching — read before your first commit

Full rules in [`.ai/GIT_WORKFLOW.md`](.ai/GIT_WORKFLOW.md), enforced by CI.

```
main  <--  release/X.Y.Z  <--  feat/… fix/… docs/…
```

- Task branches come from the **open release branch** and merge back into it.
  Never from or into `main`.
- **Never push to `main`, never create a tag, never bump the version** outside a
  release branch. Merging a release branch into `main` is what tags the commit
  and publishes to Homebrew, so an unintended merge ships a release.
- Check `git branch --show-current` before committing. If it says `main`, stop
  and branch.
- Every pull request touching `src/`, `locales/` or `Cargo.toml` needs a
  `CHANGELOG.md` entry under `## [Unreleased]`.
