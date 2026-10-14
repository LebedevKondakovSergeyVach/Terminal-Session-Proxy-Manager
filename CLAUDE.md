@AGENTS.md

# Claude Code

Keep shared instructions in `AGENTS.md`. This entry point holds only Claude
Code behavior; optional plugins must follow the repository's branch, scope
and attribution rules.

## Context loading

- The first-line import loads the shared contract with this file.
- Path-scoped `.claude/rules/*.md` load when working with matching files.
- `website/CLAUDE.md` links to `website/AGENTS.md`; edit the target.
- `CLAUDE.local.md` is git-ignored personal context, when present.
- `SESSION.md` is a local working agreement; read it when present unless its
  full content is already in context.
- Reference docs and skill bodies are on demand. Do not eagerly import them
  or reread files already present in full merely to prove that they loaded.

Use `/context` or `/memory` to inspect loaded instructions when troubleshooting.
Loading behavior is documented in the official
[Claude Code memory reference](https://code.claude.com/docs/en/memory).

## Task execution

- Keep a todo list for multi-step work.
- Use `AskUserQuestion` for material choices, recommended option first.
- Use focused subagents for independent reading and checks. Brief them with
  permitted files, read/write scope, branch and attribution rules; request
  concise evidence and spot-check it before reporting.
- Use project skills from `.claude/skills/`; the trigger map is in `AGENTS.md`.
- `.claude/settings.json` disables commit/PR/session attribution. Remove any
  attribution still proposed by a plugin; hooks enforce the commit rules.

## Optional plugins and MCP

Installed plugins are user configuration, not repository dependencies. Check
availability when a task needs one; do not maintain an installed-plugin or
model inventory here. Use a matching plugin when available and state a missing
capability instead of claiming to have invoked it.

For process plugins such as `superpowers`, keep plans in `.ai/plans/`. Their
worktree/branch/commit defaults cannot authorize switching branches, merging,
publishing or adding attribution against `AGENTS.md`. Project skills own this
repository's verification and release procedures.

`.mcp.json` declares the documentation servers. Check `/mcp` when tools are
missing. If a plugin already supplies context7, use one instance; do not enable
duplicate servers. Keep credentials and personal server settings out of
tracked files.
