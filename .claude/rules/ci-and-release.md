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
  `publish` job, publishes bilingual notes from both changelogs and bumps
  the Homebrew tap. Both changelogs must have the release section.
  The `crates` job runs after that and uploads the crate (`cargo publish
  --locked`). Its token is the `CARGO_API_KEY` secret, mapped to
  `CARGO_REGISTRY_TOKEN`. That job's `contents: read` cannot push a tag or
  edit the GitHub release, so a failed upload is retried by re-running `crates`
  alone. A full re-dispatch on the tag repeats `publish`. Details:
  `.ai/GIT_WORKFLOW.md`, "If the release job fails".
- `website.yml` — generator tests, site build, link validation, a diff check
  after content regeneration, `npm audit`. The diff does not detect unchanged
  tracked generated pages; local verification checks the manifest separately.
- `pages.yml` — publishes the site from `main`. Its `workflow_dispatch` can
  deploy another ref; treat that as a deliberate act.

Never point `pages.yml` or `release.yml` at a release or task branch, and never
widen a workflow's `permissions` beyond what a step demonstrably needs.

## Crate identity

The package, library and installed command share one name:

| Role | Name |
| :--- | :--- |
| crates.io package (`cargo install`, `cargo uninstall`) | `tspm` |
| Binary target, shell init, Homebrew, release archives | `tspm` |
| Rust library import path | `tspm` |

`tspm.io` is the project domain. It does not serve the documentation site, so
`homepage` and `documentation` stay on the GitHub Pages site. Do not point
them at `tspm.io` until that host serves these pages.

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
