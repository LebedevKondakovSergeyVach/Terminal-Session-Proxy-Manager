---
name: verify-project
description: Use before claiming any change is done, fixed, passing or ready to commit - Rust code, locales, shell or config files, dependencies, documentation, changelogs or the website.
---

# verify-project

## 1. The crate — always

```bash
cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked
```

CI adds `--all-features` to clippy and tests and supplies `-D warnings` through
`RUSTFLAGS`. `Cargo.toml` declares no `[features]` table, so the two agree.
It also runs an MSRV job on `rust-version`; if you used a newly stabilised std
API, say that the MSRV job is the one to watch.

## 2. The website — when the change touched its sources

Required when the change touched any of:
`website/`, `docs/`, `README.md`, `README.ru.md`, `CONTRIBUTING.md`,
`CHANGELOG.md`, `CHANGELOG.ru.md`, `assets/`

```bash
cd website && npm ci && npm test && npm run build
```

`npm run build` regenerates every page and fails on a dead internal link. The
same gate runs in `.github/workflows/website.yml`. A release always qualifies.

After the build, `git status --short website/src/content/docs` must show nothing
new: generated pages are ignored, and a tracked one fails CI.

## 3. What the gate does not cover

Check these by hand when they apply:

- `shell/*.zsh`, `shell/*.bash` changed → `zsh -n` / `bash -n` on them (tests
  check only the `init` output).
- `src/`, `locales/`, `shell/`, `configs/` or `Cargo.toml` changed → entries in
  **both** changelogs. Direct commits to the release branch are not checked by CI.
- Agent instruction files changed → the `agent-docs-audit` skill.

## 4. Reporting

Paste the tail of each command with its exit status. Name every check you did
not run and why. If something fails, report the error — do not describe the
task as finished.
