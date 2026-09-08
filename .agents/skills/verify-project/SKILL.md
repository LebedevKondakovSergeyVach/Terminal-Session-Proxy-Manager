---
name: verify-project
description: >-
  Verifies the project: formatting, lints and tests for the Rust crate, plus the
  documentation site whenever the change touched anything the site is built from.
---

# verify-project

## 1. The crate — always

Run these three in sequence:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --locked -- -D warnings`
3. `cargo test --locked`

CI adds `--all-features` to the clippy and test steps and supplies `-D warnings`
through the `RUSTFLAGS` environment variable rather than the flag. `Cargo.toml`
declares no `[features]` table, so the two forms agree today.

## 2. The website — when the change touched its sources

The site in `website/` is **generated** from the repository's own Markdown. So
this step is required whenever the change touched any of:

`website/`, `docs/`, `README.md`, `README.ru.md`, `CONTRIBUTING.md`,
`CHANGELOG.md`, `CHANGELOG.ru.md`, `assets/`

```bash
cd website && npm ci && npm test && npm run build
```

`npm run build` regenerates every page and fails on a dead internal link, so a
link to a document with no manifest entry stops it. This is the same gate
`.github/workflows/website.yml` runs on the pull request — running it locally
is how you find out before CI does.

A release always qualifies: both changelogs are published pages.

## 3. Reporting

If anything fails, report the error and offer to fix it. Do not describe a task
as finished until every applicable check above has passed — and say which ones
you actually ran, rather than implying all of them.
