# Verification reference

[`AGENTS.md`](../AGENTS.md) defines the required gates;
[`verify-project`](../.claude/skills/verify-project/SKILL.md) owns the procedure
and reporting. Run it before claiming a change is complete.

## Iteration commands

```bash
cargo test --lib             # library unit tests
cargo test --test cli        # tests spawning the real binary
cargo test <name-fragment>   # a focused regression
cargo run -- <subcommand>    # local behavior check; may read/write user state
```

Use isolated config/settings paths for manual state-changing checks. The
binary-test helper already isolates both. Avoid tests that require external
network access.

## Find the owning procedure

| Change | Reference |
| :--- | :--- |
| Rust, locales, config precedence, regression tests | [Rust rule](../.claude/rules/rust-source.md) |
| Shell syntax or shipped defaults | [Shell/config rule](../.claude/rules/shell-and-configs.md) |
| Bilingual docs, changelogs, screenshots | [Docs rule](../.claude/rules/docs-sources.md) |
| Site generator, styling or interactive behavior | [Website contract](../website/AGENTS.md) and [skill](../.claude/skills/starlight-website/SKILL.md) |
| CI triggers and release recovery | [Git workflow](GIT_WORKFLOW.md) |
| Agent instruction structure and references | [Instruction audit](../.claude/skills/agent-docs-audit/SKILL.md) |

CI also builds crate documentation, tests on Linux/macOS and checks the MSRV
declared by `rust-version`. The local gate does not reproduce every CI job;
state which additional checks were run instead of implying full CI coverage.
