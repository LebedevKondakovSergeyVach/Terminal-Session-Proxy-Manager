## What changed

<!-- One or two sentences. Link the issue this closes, if any. -->

## Why

<!-- The problem this solves. For a fix, what went wrong before. -->

## Checklist

- [ ] Branched from the open `release/X.Y.Z` and targeting that same branch
      (only `release/*` and `hotfix/*` may target `main` — see
      [`.ai/GIT_WORKFLOW.md`](../.ai/GIT_WORKFLOW.md))
- [ ] `cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`
      passes (CI also adds `--all-features`; the crate defines no features)
- [ ] New user-facing strings go through `I18n::t` / `I18n::format` and exist in
      **both** `locales/en.json` and `locales/ru.json`
- [ ] New or changed CLI commands are reflected in `README.md`, `README.ru.md`
      and `docs/`
- [ ] `CHANGELOG.md` **and** `CHANGELOG.ru.md` have an entry under
      `## [Unreleased]` (required for any change to `src/`, `locales/`,
      `shell/`, `configs/` or `Cargo.toml`)
- [ ] If this touches `docs/`, the READMEs, `CONTRIBUTING.md`, a changelog, `assets/` or `website/` —
      those are the website's content — `cd website && npm ci && npm test && npm run build`
      passes, and the sources are still plain CommonMark (no JSX, no `import`)

<!--
Releasing? If this is release/X.Y.Z -> main, merging publishes: CI builds four
targets, tags the commit and bumps the Homebrew tap. Confirm first that
Cargo.toml and Cargo.lock are at X.Y.Z, and that CHANGELOG.md and CHANGELOG.ru.md
both have a `## [X.Y.Z]` section with updated link footers. CI checks only
CHANGELOG.md.
-->
