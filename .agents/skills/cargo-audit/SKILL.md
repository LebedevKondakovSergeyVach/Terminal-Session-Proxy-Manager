---
name: cargo-audit
description: >-
  Automates checking for security vulnerabilities in Rust dependencies and fixing them.
---

# cargo-audit

When asked to audit or secure dependencies, execute the following steps:

1.  **Run Audit**: Execute `cargo audit`. (If it is not installed, install it via `cargo install cargo-audit`).
2.  **Analyze Report**: Review the output for any reported vulnerabilities in dependencies.
3.  **Update Dependencies**:
    - For minor/patch updates, run `cargo update -p <crate_name>`.
    - If a major version bump is required, modify `Cargo.toml` to the safe version, then run `cargo check` and `cargo test`.
4.  **Verify**: Re-run `cargo audit` to confirm 0 vulnerabilities, then invoke the `verify-project` skill — a dependency bump can break the build or the lints even when the advisory is gone.

5.  **Record it for users**: a CVE fix is something people upgrading want to know about. Add a `### Security` entry to `CHANGELOG.md` **and** `CHANGELOG.ru.md` under `## [Unreleased]`, naming the advisory and the crate.

    Note that CI will not ask you for this: `branch-policy.yml` only demands a changelog entry when `src/`, `locales/`, `shell/`, `configs/` or `Cargo.toml` changed, and a `cargo update -p` touches only `Cargo.lock`. The gap is yours to close.

6.  **Commit**: Detail which dependencies were updated and which advisories were resolved. Follow the repository's commit rules in [`AGENTS.md`](../../../AGENTS.md) — Conventional Commits, and **no authorship attribution of any kind** in the message.

    ```bash
    git commit -m "fix(deps): update <crate> to <version> for RUSTSEC-YYYY-NNNN"
    ```

    Note this repository audits on a schedule too: `ci.yml` runs `rustsec/audit-check` weekly, so an advisory can appear with no code change behind it.
