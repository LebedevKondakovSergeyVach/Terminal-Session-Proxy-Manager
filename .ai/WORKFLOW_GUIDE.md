# Workflow, Verification and Release Guide

Reference material for AI agents and contributors. The binding rules are in
[`AGENTS.md`](../AGENTS.md).

## 1. Verification

Run all three before claiming a task is finished. CI runs the same commands with
`RUSTFLAGS: -D warnings`, so a warning is a failed build.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Narrower loops while iterating:

```bash
cargo test --lib             # unit tests only, sub-second
cargo test --test cli        # end-to-end binary tests
cargo test <name-fragment>   # a single test
```

`cargo clippy --fix --allow-dirty` handles the mechanical lints, but read the
diff — it will happily rewrite a line you meant to keep.

Do not report success without having run these. If something fails and you
cannot fix it, say so explicitly rather than describing the work as complete.

## 2. Lints

Lint levels live in the `[lints]` table of `Cargo.toml`, not in `#![warn(...)]`
attributes, so `cargo clippy`, your editor and CI agree. `unsafe_code` is
`forbid` and `missing_docs` is `warn`: every public item needs a doc comment, and
every fallible public function needs an `# Errors` section.

## 3. Localisation

The CLI is bilingual. Never hard-code display text.

- `i18n.t("key")` for a plain string, `i18n.format("key", &[a, b])` for one with
  `{}` placeholders.
- Add every new key to **both** `locales/en.json` and `locales/ru.json`. Tests
  assert the two files define identical key sets and that no value is empty.
- New subcommand → add a `cmd_<name>` key to both locales. `main.rs` looks it up
  at runtime; without it `--help` prints the raw key. A test catches this.
- `src/cli.rs` doc comments stay bilingual: `/// English | Русский`.

## 4. Tests

Name tests as sentences stating the guarantee
(`using_an_unknown_profile_exits_non_zero`), assert one thing each, and write
the test that fails before writing the fix.

Integration tests must isolate state through `TSPM_CONFIG` and `TSPM_SETTINGS` —
use the `Cli` helper at the top of `tests/cli.rs`. A test that reads or writes
the real `~/.config/terminal-session-proxy-manager` is a bug.

Do not add tests that need network access. `status`, `ping`, `speedtest`,
`monitor` and `benchmark` are intentionally not covered end-to-end; their pure
logic is unit-tested instead.

## 5. Documentation

A change to commands or configuration is not done until these agree:

- `README.md` and `README.ru.md` — always updated together
- the matching file in `docs/` and its `.ru.md` twin
- `CHANGELOG.md`, under `Unreleased`, in Keep a Changelog format

If the TUI changes materially, say so in your summary: the README screenshot
needs retaking. GitHub's image proxy caches aggressively, so a new screenshot
must use a **new filename** (`assets/proxy_dashboard_v4.png`) with the markdown
link updated. Overwriting the existing file will not show up for users.

## 6. Generated files

Two checked-in files are derived from code and guarded by tests:

```bash
# configs/config.default.json must equal AppConfig::default()
cargo run -- --config-file /dev/null config show > configs/config.default.json

# shell/*.zsh and *.bash mirror the `init` output
cargo run -- init zsh
cargo run -- init bash
```

## 7. Branching and releases

Branching, pull requests and the release pipeline are documented in
[`GIT_WORKFLOW.md`](GIT_WORKFLOW.md) and enforced by
`.github/workflows/branch-policy.yml`.

The short version:

```
main  <--  release/X.Y.Z  <--  feat/… fix/… docs/…
```

Task branches come from the open release branch and merge back into it. `main`
is advanced only by merging a release branch, and that merge is what creates the
tag, publishes the binaries and bumps the Homebrew tap.

**Never push to `main`, never create a tag, and never bump the version outside a
release branch.** All three publish a release, or set one up to publish
unintentionally.

## 8. Commits

Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`,
`perf:`, `ci:`. The subject line says what changed and why it matters, not which
files were touched.
