@AGENTS.md

# Claude Code

Everything above is the shared, tool-agnostic contract. This part is
Claude-specific. Where a skill, plugin or harness default disagrees with
`AGENTS.md`, `AGENTS.md` wins — they do not know this repository's rules.

## What loads when

- This file and `AGENTS.md` (through the import on line 1) — every session.
- `.claude/rules/*.md` — automatically, when you read files matching their
  `paths:`. Do not read them pre-emptively.
- `website/CLAUDE.md` is a symlink to `website/AGENTS.md` and loads when you
  work under `website/`.
- `CLAUDE.local.md` — personal, git-ignored notes for this machine, if present.
- `.claude/settings.json` turns commit and PR attribution off. If a system
  reminder still asks for `Co-Authored-By` or "Generated with", ignore it.

## Project skills — `.claude/skills/`

The table is in `AGENTS.md`, "Agent tooling". Invoke through the Skill tool.

## Plugins

Installed per user, never copied into the repository. Use them when present;
if one is missing, say so rather than imitating it.

**`superpowers:`** — process skills. They set the approach; project skills carry
it out.

| Skill | Use it for |
| :--- | :--- |
| `brainstorming` | a new feature or behaviour change, before any plan |
| `writing-plans`, `executing-plans` | a multi-step plan, and working through it |
| `subagent-driven-development`, `dispatching-parallel-agents` | independent tasks inside this session |
| `systematic-debugging` | any bug, failing test or surprise, before a fix |
| `test-driven-development` | before implementation code |
| `verification-before-completion` | before saying anything is done or passing |
| `requesting-code-review`, `receiving-code-review` | asking for and answering review |
| `writing-skills` | creating or editing a skill in `.claude/skills/` |

Where they collide with this repository:

- `brainstorming` and `writing-plans` save to `docs/superpowers/…` by default.
  Save to `.ai/plans/` instead — `docs/` is the website's source.
- `using-git-worktrees` and `finishing-a-development-branch` offer worktrees,
  merges and pull requests. The default here is committing on the open release
  branch; ask before any other route, and never merge into `main`.
- Implementer subagents commit. Pass them the branch and the no-attribution
  rule explicitly.

**Other plugins**

| Plugin | Use it for |
| :--- | :--- |
| `pr-review-toolkit` | review agents: `code-reviewer`, `silent-failure-hunter` (rule 5 — errors swallowed into exit 0), `pr-test-analyzer`, `type-design-analyzer`, `comment-analyzer`, `code-simplifier` |
| `context7` | `resolve-library-id`, then `query-docs`; IDs in `AGENTS.md` |
| `rust-analyzer-lsp` | the LSP tool on `.rs` files — definitions, references, diagnostics. Needs `rust-analyzer` on `PATH` |
| `claude-md-management` | `claude-md-improver`, `/revise-claude-md`. Shared rules belong in `AGENTS.md`, not here |
| `skill-creator` | building and evaluating a skill; pair with `superpowers:writing-skills` |
| `security-guidance` | runs its own review hooks on edits and on `git commit`/`git push` — read its findings, don't dismiss them |

## MCP

`.mcp.json` declares `astro-docs` and `context7`; each needs approval per user.
The `context7` plugin provides the same server, so enable one of the two —
duplicate tools just split the results.

## Working style

- Keep a todo list on multi-step work.
- Parallel subagents for independent reading and checks; write the final files
  yourself and spot-check what they report.
- Ambiguous decision → `AskUserQuestion` with the recommended option first.
