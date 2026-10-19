---
name: cursor-delegate
description: Use when research, a read-only re-check or an independent review should run on headless Cursor Grok 4.7 (cursor-agent) - before reporting a finished work chunk, for a Grok QUESTION to answer, choosing effort, 256K or 500K context or Fast, parallel Grok runs, a stuck or failed cursor-agent run, "Cannot use this model".
---

# Delegating to headless Cursor Grok 4.7

`run.sh` starts `cursor-agent -p --output-format stream-json` on a brief in the background, writes a
short digest while it works, resumes the same session for an answer, and checks which model ran.
Briefs live in git-ignored `.ai/briefs/NN-<slug>.md`; the agent writes `NN-<slug>.result.md` next to
the brief, logs land in `.ai/briefs/logs-NN/`.

## When to use

- Large but checkable work: reading many files, comparing projects, research, a review of a diff,
  the read-only re-check every finished chunk gets before it is reported as verified.
- Not for: decisions that are the user's, results you cannot verify, git mutations, commits,
  pushes or GitHub writes. Those stay with the user and the parent session.
- Manual path: `cursor-agent status` shows it logged out, the run needs an approval mid-way, or the
  user asks. Tell the user which brief to run and wait.

## Quick reference

Run from the repository root; start and answer with `run_in_background: true`.

| Step | Command |
| :--- | :--- |
| Start with policy defaults | `bash .claude/skills/cursor-delegate/run.sh .ai/briefs/NN-<slug>.md --mode ask` |
| Start with other settings | `... run.sh <brief> [--effort low\|medium\|high\|xhigh] [--context 256k\|500k] [--fast] [--mode plan\|ask] [--strict]` |
| Watch progress | `tail -n 30 .ai/briefs/logs-NN/NN-<slug>.log` |
| Final message | `.ai/briefs/logs-NN/NN-<slug>.last.md` |
| Answer a `QUESTION:` | `bash .claude/skills/cursor-delegate/run.sh <brief> --answer "<text>"` |
| Live Grok 4.7 ids | `bash .claude/skills/cursor-delegate/run.sh --models` |

## Model settings

The policy owner is the local `SESSION.md`. Without it, use these defaults; they are `run.sh`'s, so a
policy run passes no model flag. Anything else only on the user's request in this session.

| Setting | Flag | Values | Default |
| :--- | :--- | :--- | :--- |
| Effort | `--effort` | `low`, `medium`, `high`, `xhigh` | `xhigh` |
| Context | `--context` | `256k`, `500k` | `256k` |
| Fast | `--fast` | on when present | off |

`run.sh` passes `--model 'grok-4.7[context=<C>,reasoning_effort=<E>,fast=<true|false>]'`, the only
form that selects 500K; flat ids such as `grok-4.7-xhigh` run with 256K. `effort=` instead of
`reasoning_effort=` fails with `Cannot use this model` (exit 1).

The digest line `SESSION <id> model=<label>` shows what ran, e.g. `Grok 4.7 256K Extra High`.
`run.sh` stores the expected label in `NN-<slug>.expect` and prints
`[WARN] model ran as '...', requested '...'` on a mismatch: stop and tell the user.

The brief's first line states the settings and why:
`Model: Grok 4.7, xhigh, 256K, Fast off (run.sh defaults) - <why>`.

## Writing the brief

1. Model line as above.
2. Goal and why it is needed.
3. Verified facts with `path:line`; separately, what is not verified.
4. Files to read first: `AGENTS.md`, the path rules and skills the work needs.
5. Scope: readable paths, the one writable result file, forbidden actions.
6. Steps.
7. Stop condition: last line `QUESTION: <question with options>`.
8. Every shell command it needs, one per call. The guard and Cursor's auto-review classifier
   reject what they cannot classify (chained commands, pipelines), and nobody can approve a
   rejection in a headless run.
9. Done criteria and how to check them.
10. Output: path, sections, length limit.

For a review, ask for high-confidence findings only, each with severity, `path:line`, evidence and a
concrete fix; regressions separate from existing issues; no cosmetic findings.

## Modes and parallel runs

- `--mode ask`: research, reviews, re-checks. No file edits except markdown (the result file).
  If Grok asks whether it may write the result, answer that markdown is allowed (`--answer`).
- `--mode plan`: the same for planning. No `--mode`: the agent may edit files; use it only for briefs
  whose scope names those files.
- Parallel runs: one brief number each, each started in its own background call, disjoint outputs.
- Do not edit `run.sh` or `digest.py` while a run uses them; bash reads the script as it runs.

## Permissions and guards

- `run.sh` sets `CURSOR_DELEGATE=1`, which enables `.cursor/hooks/delegate-guard.sh`. It splits the
  command like a shell (quotes, chains, `$(...)`, `bash -c`, `/usr/bin/` paths) and denies git
  mutations including `fetch`, `git branch` and `git config` writes, `gh` writes and `gh api`,
  `cargo publish|install|login` (except `--dry-run`), `npm publish|login` and recursive `rm`. It
  fails closed. Cases: `.cursor/hooks/tests/delegate-guard-test.sh`. A user-level
  `~/.cursor/hooks/guard-git.sh`, when installed, runs as well. The guards catch mistakes, not
  deliberate evasion; a brief that reads untrusted content (web pages, issues) still runs with the
  user's git and GitHub credentials.
- Default `--auto-review`: a Cursor classifier runs commands it judges safe; the guard is what
  blocks mutations. `--strict` drops it, leaving only `permissions.allow` of `~/.cursor/cli-config.json`.
- Never `--force` or `--yolo`. Never add a project `.cursor/cli.json`: its `permissions.allow`
  replaces the user's list in the user's own sessions too.
- Headless limits: WebSearch is rejected; give documentation through Context7 or name the URLs.
  Context7 for Cursor is the user-level server configured with the API key (local `SESSION.md`).

## Supervising

1. Read only the digest (`.log`, `.last.md`), never the `.jsonl`: it carries reasoning deltas.
2. Digest lines: `SESSION`, `SAY`, `SHELL ok|success|failure|rejected <cmd>`, `READ`/`EDIT`/`GREP`,
   `DONE <status> <s> in/out/cache`, `QUESTION <text>`, closing `=== exit <code>` with the result
   file's presence. `[WARN] git branch or HEAD changed`: the run touched git; `[WARN] guard or hook
   files changed`: the run edited `.cursor/`, `.claude/hooks/` or the runner. Stop, tell the user.
   `--answer` refuses a `.args` file holding anything but the flags a start writes.
3. `QUESTION`: answer from the brief, `AGENTS.md` or the code; a decision that is the user's goes
   to them first (Claude Code: AskUserQuestion). Then `--answer`.
4. Stuck run: stop its background task, or `pkill -f 'cursor-agent.* -p .*<NN-slug>.md'` (start
   round) / `pkill -f 'cursor-agent.* -p .*<session-id>'` (answer round; the id is line 1 of
   `logs-NN/NN-<slug>.session`). Continue with `--answer`. No `.session` file: repeat the start.
5. After `DONE`: read the result, spot-check its key claims (`path:line`, `ls`), fix confirmed issues
   in the parent session, run the gates, and request a focused re-check for substantive fixes.
   Report which shell commands the run could not execute: its claims about command output are then
   unverified.

## Failures

| Symptom | Meaning, action |
| :--- | :--- |
| `grok-4.7-<E> is not in cursor-agent --list-models` | Grok renamed or removed: `run.sh --models`, tell the user; never switch family |
| `Cannot use this model: grok-4.7[...]` (exit 1) | a parameter value Cursor does not offer, or a transient list without Grok: `run.sh --models`, then repeat the start |
| `DONE error` with an auth message | `cursor-agent status`; logged out: manual path |
| `[WARN] no SESSION line` | the start died before the model ran; read the `RAW` lines, fix, repeat |
| `<...>.session exists` | that brief already ran: `--answer`, or a new brief number |
| `SHELL rejected ... Hook script timed out` | the guard did not answer within the `.cursor/hooks.json` timeout (seen in the first seconds of three parallel starts); the run continues with file tools only. Re-run the shell-dependent checks in a new brief |
| other `SHELL rejected` | the classifier or the guard; if the agent retries variants, stop the run and fix the brief |
