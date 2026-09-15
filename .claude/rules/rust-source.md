---
paths:
  - "src/**"
  - "tests/**"
  - "locales/**"
---

# Rust source, tests and locales

The invariants are rules 1-7 in `AGENTS.md`. This file is the mechanics.

## Translations

- `i18n.t("key")` for a plain string, `i18n.format("key", &[a, b])` for `{}`
  placeholders. Keys in `locales/*.json` are kept in alphabetical order.
- Tests in `src/config/i18n.rs` assert both locales define the same keys and no
  value is empty; `main.rs` asserts every subcommand has a `cmd_<name>` key.
  Without that key `--help` prints the raw key.
- `src/cli.rs` doc comments stay bilingual: `/// English text | Русский текст`.
  They are the static fallback; `main.rs` swaps in `cmd_<name>` at runtime.
- Error text inside `ProxyError` is English. Attach the translated sentence as
  `anyhow` context so the user reads their language first:

  ```rust
  return Err(ProxyError::ProfileNotFound(key.to_string()))
      .context(i18n.format("profile_not_found", &[key]));
  ```

  A new `ProxyError` variant only when a caller needs to match on it.

## A new subcommand or flag touches all of these

`src/cli.rs` (variant + bilingual doc), its handler (`src/cmd/<name>.rs` or the
dispatch in `main.rs`), `cmd_<name>` and any labels in both locales, tests,
`README.md` + `README.ru.md`, the matching `docs/*.md` + `.ru.md`, and both
changelogs. Flag names and exit codes are interface: renaming one is not a
patch release (`.ai/GIT_WORKFLOW.md`, "Choosing the number").

## Tests

- Named as sentences stating the guarantee
  (`using_an_unknown_profile_exits_non_zero`); one assertion each.
- Pure logic: `#[cfg(test)] mod tests` beside the code. Argv, exit codes, files:
  `tests/cli.rs`.
- Integration tests isolate state through `TSPM_CONFIG`, `TSPM_SETTINGS` and
  `TSPM_LANG` — use the `Cli` helper at the top of `tests/cli.rs`. A test that
  touches the real `~/.config` is a bug.
- No network. `status`, `ping`, `speedtest`, `monitor` and `benchmark` are
  deliberately not covered end-to-end; unit-test their pure logic.
- A bug fix comes with the test that fails without it.

## Lints and docs

Lint levels live in `[lints]` of `Cargo.toml`: `unsafe_code = "forbid"`,
`missing_docs = "warn"` (an error under CI's `-D warnings`). An `# Errors`
section on fallible public functions is a convention — no lint enforces it.

## Configuration precedence

Know this before changing path resolution:

- `config.json`: `--config-file` → `TSPM_CONFIG` → `config_path` in
  `settings.json` → OS config dir.
- `settings.json`: `--settings-file` → `TSPM_SETTINGS` → OS config dir →
  `./settings.json` in the working directory (last on purpose).
- Language: `--lang` → `TSPM_LANG` → `lang` in `settings.json` → `ru`.

`--config-file`, `--settings-file` and `--lang` are read from raw argv by
`main::preparse` before clap parses, because help text is localised while the
command tree is built.

## Library APIs

Look crate APIs up through context7 (IDs in `AGENTS.md`, "Agent tooling")
rather than recalling them. Check the version in `Cargo.toml` first and prefer a
version-pinned ID when one exists (`/websites/rs_ratatui_0_30_0`).
