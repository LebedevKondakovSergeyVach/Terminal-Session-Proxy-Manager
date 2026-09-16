---
paths:
  - "shell/**"
  - "configs/**"
---

# Shell integration and shipped defaults

Both directories are user-facing: a change here needs an entry under
`## [Unreleased]` in `CHANGELOG.md` **and** `CHANGELOG.ru.md`.

## `shell/`

- The `proxy` shell function for zsh and bash is the documented integration
  point, and `docs/SHELL_INTEGRATION*.md` describe it.
- These files are **hand-maintained**. `src/cmd/init.rs` emits its own copy of
  the function; no test compares the two, so a change to one must be mirrored in
  the other by hand. `cargo run -- init zsh` prints to stdout — it does not
  write `shell/`.
- Whatever the function passes to `eval` comes from `env on` or the dashboard
  hand-off file (`src/shell_handoff.rs`), whose values are already quoted by
  `proxy_env::shell_quote`. Do not add a second, unquoted path.
- Keep it valid in both shells: `tests/cli.rs` syntax-checks the `init` output,
  not these files. Check them yourself with `zsh -n` and `bash -n`.

## `configs/config.default.json`

Must equal `AppConfig::default()`; the test
`the_shipped_default_config_matches_the_code` in `src/config/app.rs` compares
them. Never edit it by hand — regenerate:

```bash
cargo run -- --config-file /dev/null config show > configs/config.default.json
```

`/dev/null` fails to parse, so a warning goes to stderr; stdout is still the
defaults. Defaults reference only loopback and well-known public endpoints.
