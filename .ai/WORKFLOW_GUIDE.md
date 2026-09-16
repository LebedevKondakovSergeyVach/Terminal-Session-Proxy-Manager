# Workflow, Verification and Release Guide

Reference material for AI agents and contributors. The binding rules are in
[`AGENTS.md`](../AGENTS.md).

## 1. Verification

Run the gate before claiming a task is finished. A warning is a failed build:
`ci.yml` sets `RUSTFLAGS: -D warnings`, and clippy here is given `-D warnings`.

```bash
cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked
```

CI additionally passes `--all-features` to clippy and the tests. `Cargo.toml`
has no `[features]` table, so that flag changes nothing you need to reproduce.

Narrower loops while iterating:

```bash
cargo test --lib             # unit tests only, sub-second
cargo test --test cli        # end-to-end binary tests
cargo test <name-fragment>   # a single test
```

`cargo clippy --fix --allow-dirty` handles the mechanical lints, but read the
diff — it will happily rewrite a line you meant to keep.

### The website

The site in `website/` is generated from `README*.md`, `docs/*.md`,
`CONTRIBUTING.md` and `CHANGELOG*.md`, with images from `assets/`. A change to
any of those, or to `website/` itself, is a change to the site, so verify it too:

```bash
cd website && npm ci && npm test && npm run build
```

`npm ci` installs exactly what `package-lock.json` pins (`npm install` is fine
while iterating), `npm test` runs the content generator's unit tests, and
`npm run build` regenerates every page and fails on a dead internal link.

`.github/workflows/website.yml` runs the same commands on pull requests and on
pushes to `main`, `release/**` and `hotfix/**` touching those paths, and
`.github/workflows/pages.yml` publishes on pushes to `main`.

Do not report success without having run these. If something fails and you
cannot fix it, say so explicitly rather than describing the work as complete.

## 2. Lints

Lint levels live in the `[lints]` table of `Cargo.toml`, not in `#![warn(...)]`
attributes, so `cargo clippy`, your editor and CI agree. `unsafe_code` is
`forbid` and `missing_docs` is `warn`: every public item needs a doc comment, and
a missing one fails the gate.

Every fallible public function also gets an `# Errors` section. That one is a
convention, not a lint: `[lints.clippy]` enables the `all` group (plus a few
named lints), and `clippy::missing_errors_doc` is not in it, so nothing fails
when the section is absent. Reviewers check it.

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

Integration tests must isolate state through `TSPM_CONFIG`, `TSPM_SETTINGS` and `TSPM_LANG` —
use the `Cli` helper at the top of `tests/cli.rs`. A test that reads or writes
the real `~/.config/terminal-session-proxy-manager` is a bug.

Do not add tests that need network access. `status`, `ping`, `speedtest`,
`monitor` and `benchmark` are intentionally not covered end-to-end; their pure
logic is unit-tested instead.

## 5. Documentation

A change to commands or configuration is not done until these agree:

- `README.md` and `README.ru.md` — always updated together
- the matching file in `docs/` and its `.ru.md` twin
- `CHANGELOG.md` **and** `CHANGELOG.ru.md`, under `Unreleased`, in Keep a
  Changelog format — both are published pages on the site. On a pull request
  into `release/*` CI rejects an English entry without its Russian twin; a
  direct commit is not checked, so that is on you

If the TUI changes materially, say so in your summary: the README screenshot
needs retaking. GitHub's image proxy caches aggressively, so a new screenshot
must use a **new filename** with the markdown link updated in both READMEs —
the current one is `assets/proxy_dashboard.png`, so a replacement might be
`assets/proxy_dashboard_v2.png`. Overwriting the existing file will not show up
for users.

## 6. Files derived from code

Two kinds of checked-in file restate what the code produces. Only one of them is
guarded by a test.

**`configs/config.default.json`** must equal `AppConfig::default()`. The test
`the_shipped_default_config_matches_the_code` in `src/config/app.rs` fails when
they differ. Regenerate it with:

```bash
cargo run -- --config-file /dev/null config show > configs/config.default.json
```

**`shell/terminal-session-proxy-manager.zsh`** and its `.bash` twin are
hand-maintained. Their headers say they were generated from
`terminal-session-proxy-manager init zsh` (or `bash`), but no test or source
file reads `shell/`, so nothing catches drift. They hold the shell function that `src/cmd/init.rs` prints, minus
the completion script `init` also emits. When you change the function in
`init.rs`, update both files by hand to match. To see the current output:

```bash
cargo run -- init zsh     # prints to stdout; it does not write shell/
cargo run -- init bash
```

## 7. Branching and releases

Branching, pull requests and the release pipeline are documented in
[`GIT_WORKFLOW.md`](GIT_WORKFLOW.md) and, for pull requests, enforced by
`.github/workflows/branch-policy.yml`.

The short version:

```
main  <--  release/X.Y.Z  <--  feat/… fix/… docs/…
```

Work lands on the open release branch: the maintainer commits to it directly,
and contributors — or changes the maintainer wants reviewed — come in through a
task branch and a pull request into it. `main` is advanced only by merging a
release branch, and that merge is what builds the binaries, creates the tag,
publishes and bumps the Homebrew tap. A direct commit to the release branch is
not checked by `branch-policy.yml`, so its changelog rule is kept by hand.

**Never push to `main`, never create a tag, and never bump the version outside a
release branch.** All three publish a release, or set one up to publish
unintentionally.

## 8. Commits

Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`,
`perf:`, `ci:`, `build:`. The subject line says what changed and why it matters,
not which files were touched.
