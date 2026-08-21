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
4.  **Verify**: Re-run `cargo audit` to confirm 0 vulnerabilities.
5.  **Commit**: Create a commit detailing which dependencies were updated and which CVEs were resolved.
