# GEMINI.md

Entry point for Antigravity (`agy`) and other Gemini-based agents. Antigravity
loads `GEMINI.md` and `AGENTS.md` from the working directory up to the
repository root, so `AGENTS.md` — the shared contract — is already in your
context. Follow it. This file adds only what is specific to you, and the
anti-fabrication contract that every agent here works under.

## 0. Before the first tool call

1. `AGENTS.md` is binding, and so is `website/AGENTS.md` when you work under
   `website/`.
2. Open the skill that covers the task: `.agents/skills/<name>/SKILL.md`
   (symlinks to `.claude/skills/`, the same files). Map: `AGENTS.md`,
   "Agent tooling".
3. Open the path-scoped rule for the files you will touch, from the table in
   `AGENTS.md`, "Where the detail lives". Antigravity does not load
   `.claude/rules/` by itself.

Start your first reply with what you actually read, so a skipped read shows:

```
Read: AGENTS.md, .agents/skills/verify-project/SKILL.md, .claude/rules/rust-source.md
```

If a file does not exist, say so instead of describing it. After a context
reset, read again — a summary of a file is not the file.

## 1. Anti-fabrication contract

State only what you observed in this session. Everything else is a question.

Not allowed:

- Naming a file, type, function, test, flag, workflow job or setting you have
  not seen in tool output this session.
- Quoting an error, log line or version number from memory.
- Saying a build, test or command passed or failed without its output in the
  session. "Should work" and "tests are green" are not reports.
- Presenting a plan's expected result as a finished result.

Every factual claim carries either `path/to/file.rs:123` with the quoted line,
or the command you ran and its output. When the data is not enough, stop and
write:

```
NOT ENOUGH DATA: <what exactly is unknown>. Needed: <command, file or access>.
```

## 2. Tools

- Read and search with your native file tools (`view_file`, `grep_search`,
  `list_dir`, `find_by_name`) rather than `cat`, `grep` or `ls`.
- Run through the shell what the docs prescribe: `cargo`, `npm`, `git`, `zsh -n`.
  A command quoted in `AGENTS.md`, a rule or a skill is meant to be run as
  written.
- MCP: Antigravity reads servers from `~/.gemini/config/mcp_config.json` (user
  scope) and from plugins; the repository's `.mcp.json` is Claude Code's format.
  context7 and astro-docs usage rules: `AGENTS.md`, "Agent tooling".

## 3. Git

`AGENTS.md`, "Branching" and "Conventions". The short form: commit on the open
`release/X.Y.Z` branch (a task branch and pull request only when the maintainer
asks), never push to `main` or push a tag, and no
authorship lines of any kind — no `Co-Authored-By`, no "Generated with Gemini"
or "Antigravity". A mobile or remote approval authorises one command, not the
git rules above.

## 4. Before you call a task done

List what you actually did, and name what you skipped:

- the commands you ran, with exit status;
- for a code change, the gate from `AGENTS.md` with its output;
- for a docs change that feeds the site, `cd website && npm ci && npm test && npm run build`;
- what you did **not** verify.

An honest "builds, tests not run" beats a confident "works".
