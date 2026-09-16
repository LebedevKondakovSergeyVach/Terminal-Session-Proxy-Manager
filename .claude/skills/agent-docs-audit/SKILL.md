---
name: agent-docs-audit
description: Use when asked to check, audit, refresh or clean up the agent instruction files (AGENTS.md, CLAUDE.md, GEMINI.md, .claude/rules, .claude/skills, .cursor/rules, .ai/), after editing any of them, after changing CI, the verify commands or the branching model, or when a fact in them looks stale.
---

# Auditing the agent instruction set

Run the bundled script first. It is read-only and takes a second:

```bash
bash .claude/skills/agent-docs-audit/audit.sh
```

It exits non-zero on a FAIL. WARN needs a look; INFO is context.

## How the files fit together

| File | Loaded by | Role |
| :--- | :--- | :--- |
| `AGENTS.md` | Claude (via `@AGENTS.md`), Antigravity, Cursor | the contract — tool-agnostic |
| `CLAUDE.md` | Claude Code | import + Claude-only plugins and quirks |
| `GEMINI.md` | Antigravity | anti-fabrication contract, tool notes |
| `.cursor/rules/cursor.mdc` | Cursor (`alwaysApply`) | routing only |
| `.claude/rules/*.md` | Claude by `paths:`; Cursor via same-named `.mdc` pointers | area mechanics |
| `.claude/skills/*` | Claude; `.agents/skills/*` symlinks for Antigravity and Cursor | procedures |
| `website/AGENTS.md` | all, under `website/` (`website/CLAUDE.md` is a symlink) | site contract |
| `CLAUDE.local.md`, `SESSION.md` | git-ignored, personal | machine facts, working agreement |

## Reading the output

1. **Budget.** `AGENTS.md` and `CLAUDE.md` load into every Claude session.
   Over 200 lines: move area mechanics into a `.claude/rules/` file with a Cursor
   pointer, not into a new always-on file.
2. **Imports.** Line 1 of `CLAUDE.md` must be `@AGENTS.md`. Any other bare
   `@file.md` loads eagerly — wrap it in backticks.
3. **Skills.** Directory name = `name:`; the description starts "Use when" and
   states triggers only. Every skill needs its `.agents/skills/<name>` symlink,
   or Antigravity and Cursor never see it.
4. **Rules parity.** A `paths:` glob that matches nothing never loads, and
   nobody is told. The Cursor pointer must carry the same globs.
5. **Missing paths.** A backticked path that does not exist is usually a rename
   the docs missed. Some hits are illustrative — read the line before deleting.
6. **One gate.** The verify commands must be identical everywhere. A copy
   without `--locked` or `npm ci` is the one that rotted.
7. **Tool leakage.** `AGENTS.md` is shared; harness tool names belong in
   `CLAUDE.md` or `GEMINI.md`.
8. **Attribution guards.** Hooks are per clone — a fresh clone needs
   `git config core.hooksPath .githooks`. History with attribution cannot be
   rewritten on a published branch; stop it recurring instead.

## What the script cannot see

- **Truth.** It checks that files agree with each other, not with the code.
  When a claim matters — a function name, a CI trigger — open the source. Brief
  a read-only subagent for a full claim-by-claim sweep when many files changed,
  and spot-check its findings.
- **Duplication.** The same rule in two files drifts. Keep one statement and
  point to it.
- **Volatile facts.** Versions, IDs and installed tools rot. Date them, and keep
  machine-specific ones in `CLAUDE.local.md`, never in a tracked file.

## Repairing

Fix findings in place, re-run the script, and confirm the section is clean.
Tracked instruction files are committed like any docs change (`docs:` or
`chore:`); run the website gate as well if a site source was touched.
