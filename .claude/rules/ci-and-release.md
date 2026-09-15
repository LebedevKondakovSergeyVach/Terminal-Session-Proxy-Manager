---
paths:
  - ".github/**"
  - "Cargo.toml"
  - "Cargo.lock"
---

# CI, dependencies and releases

High-risk area: a mistake here publishes. Read the workflow file before stating
what a job does — names are not behaviour. Reference: `.ai/GIT_WORKFLOW.md`,
"What CI runs, and when".

## Workflows

- `ci.yml` — fmt, clippy, docs, tests on Linux and macOS, MSRV, `cargo audit`.
  Pull requests, pushes to `main`/`release/**`/`hotfix/**`, weekly, manual.
- `branch-policy.yml` — branch names, head/base pairing, the changelog check
  (pull requests into `release/*` only), release readiness (release → `main`).
- `release.yml` — runs on any push to `main`, a pushed `v*` tag, or manual
  dispatch. Re-verifies, builds four targets, **then** creates the tag in the
  `publish` job, publishes and bumps the Homebrew tap.
- `website.yml` — generator tests, site build, link validation, no tracked
  generated pages, `npm audit`.
- `pages.yml` — publishes the site from `main`. Its `workflow_dispatch` can
  deploy another ref; treat that as a deliberate act.

Never point `pages.yml` or `release.yml` at a release or task branch, and never
widen a workflow's `permissions` beyond what a step demonstrably needs.

## Versions

- `version` in `Cargo.toml` changes only when a release is cut on
  `release/X.Y.Z`, and must match the branch name — `branch-policy.yml` compares
  them. Never as a side effect of another task. Procedure: `release-manager`.
- `rust-version` is the MSRV the CI job checks. Raising it is a user-visible
  change: changelog entry, and say why.
- A `Cargo.toml` change needs both changelog entries. A `cargo update -p` that
  only touches `Cargo.lock` does not trigger CI's check — a security fix still
  deserves a `### Security` entry (`cargo-audit` skill).
- Dependabot opens grouped updates weekly for cargo and npm (`/website`),
  monthly for GitHub Actions.

## Before you push a change here

Run the gate from `AGENTS.md`. For workflow YAML, also reread the whole job you
edited: a typo in an `if:` silently skips a job rather than failing it.
