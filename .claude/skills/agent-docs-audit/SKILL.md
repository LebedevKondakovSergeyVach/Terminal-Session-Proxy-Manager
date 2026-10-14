---
name: agent-docs-audit
description: Use when asked to check, audit, refresh or clean up agent instructions (AGENTS.md, CLAUDE.md, .claude, .cursor/rules, .ai), after editing them, after changing CI, verification or branching, or when a documented fact looks stale.
---

# Audit agent instructions

Run the read-only structural check first:

```bash
bash .claude/skills/agent-docs-audit/audit.sh
```

A FAIL exits nonzero; investigate every WARN. Passing structure does not prove
that the documented behavior matches the source.

## Ownership

| File | Role |
| :--- | :--- |
| `AGENTS.md` | shared contract, invariants, gates and navigation |
| `CLAUDE.md` | single import plus Claude-only behavior |
| `.cursor/rules/cursor.mdc` | Cursor routing to the shared contract |
| `.claude/rules/*.md` | path-specific mechanics; matching Cursor pointers |
| `.claude/skills/*/SKILL.md` | task procedures; `.agents/skills/` aliases |
| `.ai/PROJECT_OVERVIEW.md` | semantic project map, flows and change entry points |
| `.ai/GIT_WORKFLOW.md` | CI/release reference |
| `.ai/WORKFLOW_GUIDE.md` | test entry points and verification references |
| `website/AGENTS.md` | site contract; `website/CLAUDE.md` links to it |
| `.ai/plans/` | current plans only |
| `SESSION.md`, `CLAUDE.local.md` | optional git-ignored working agreement/local notes |

## Structural checks

The script checks startup size, imports, skill frontmatter/aliases, path-rule
applicability and Cursor parity, local paths/links, instruction language,
verification-command consistency and attribution guards. Local notes are
included when present, without inspecting credential or permission files.

Keep startup files compact. Reference material belongs in the map, path rules
or skills; adding an eager import does not save context. Re-resolve volatile
API references when using them. Machine tool/plugin inventories do not belong
in tracked contracts.

## Semantic review and repair

1. Check important claims directly in code or workflows: handler names,
   precedence, error paths, CI triggers, actual checks and release effects.
   For a large change, use a focused read-only review subagent; spot-check it.
2. Keep rules in one owning document. Replace duplicated procedures with links,
   preserving active invariants and their reasons. Instruction prose is English;
   bilingual product content is outside this language check.
3. Remove completed/abandoned plans after moving lasting decisions into current
   references. Do not keep old implementation snippets or transient blockers
   as instructions; Git history already records them.
4. Fix stale links and pointers when removing or renaming a file. Confirm the
   project map's named paths still exist and its flows agree with current code.
5. Rerun the script and `verify-project`. Site sources or `website/` changes also
   require the site gate. Report what was checked and any remaining limits.
