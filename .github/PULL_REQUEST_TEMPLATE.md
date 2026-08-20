## What changed

<!-- One or two sentences. Link the issue this closes, if any. -->

## Why

<!-- The problem this solves. For a fix, what went wrong before. -->

## Checklist

- [ ] Branched from the open `release/X.Y.Z` and targeting that same branch
      (only `release/*` and `hotfix/*` may target `main` — see
      [`.ai/GIT_WORKFLOW.md`](../.ai/GIT_WORKFLOW.md))
- [ ] `cargo fmt --all` leaves no diff
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` is clean
- [ ] `cargo test` passes
- [ ] New user-facing strings go through `I18n::t` / `I18n::format` and exist in
      **both** `locales/en.json` and `locales/ru.json`
- [ ] New or changed CLI commands are reflected in `README.md`, `README.ru.md`
      and `docs/`
- [ ] `CHANGELOG.md` has an entry under `## [Unreleased]`
      (required for any change to `src/`, `locales/` or `Cargo.toml`)

<!--
Releasing? If this is release/X.Y.Z -> main, merging publishes: CI tags the
commit, builds four targets and bumps the Homebrew tap. Confirm first that
Cargo.toml is at X.Y.Z and CHANGELOG.md has a `## [X.Y.Z]` section.
-->
