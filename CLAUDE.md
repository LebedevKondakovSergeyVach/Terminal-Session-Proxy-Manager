# CLAUDE.md

This project keeps its agent instructions in [`AGENTS.md`](AGENTS.md), following
the [agents.md](https://agents.md/) convention so that every tool reads the same
file. **Read `AGENTS.md` before making changes.**

Reference material it links to:

- [`.ai/PROJECT_OVERVIEW.md`](.ai/PROJECT_OVERVIEW.md) — architecture and design constraints
- [`.ai/GIT_WORKFLOW.md`](.ai/GIT_WORKFLOW.md) — branching, pull requests and releases
- [`.ai/WORKFLOW_GUIDE.md`](.ai/WORKFLOW_GUIDE.md) — verification process

The short version, if you read nothing else:

```bash
cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked
```

- Every user-facing string goes through `I18n` and exists in **both**
  `locales/en.json` and `locales/ru.json`.
- Never overwrite a config file that failed to parse.
- Everything a shell will `eval` goes through `proxy_env::shell_quote`.
- A failing command returns `Err`, never a printed message with exit code 0.
- Work on a task branch off the open `release/X.Y.Z`. Never push to `main`,
  never create a tag — merging a release branch into `main` publishes.
