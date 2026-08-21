---
name: verify-project
description: >-
  Verifies the Rust project by running formatting checks, lints, and all tests exactly as CI does.
---

# verify-project

When the user asks you to verify the project, run the following three commands in sequence:
1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --locked -- -D warnings`
3. `cargo test --locked`

If any of these fail, report the error and offer to fix it. Do not consider a task finished until all three checks pass.
