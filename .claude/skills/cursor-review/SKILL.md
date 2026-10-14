---
name: cursor-review
description: Use when asked for an independent Cursor review, for a Grok check, or before finishing a work chunk under the local session review policy.
---

# Independent Cursor review

1. Read `AGENTS.md`, the relevant path rule and local `SESSION.md`, when present.
   Check `agent models`; use the user's requested model or the session policy.
   If neither supplies a model, ask for a preference. Never silently substitute
   an older model.
2. Write a bounded brief outside published documentation. State model/effort
   and purpose, verified facts with source locations, unknowns, files to read,
   permitted paths/commands, forbidden writes, stop conditions, done criteria
   and expected report. Keep source/credential access within the task's scope.
3. Run the review in ask mode with the exact model ID from the live list:

   ```bash
   agent --mode ask --print --output-format json --model <selected-model> \
     --workspace <repository-root> --trust "Read <absolute-brief-path> and perform its read-only review."
   ```

   Keep the final response and logs outside tracked instructions. Use a process
   handle for background work, check concise progress, and avoid reading raw
   reasoning streams. `--trust` accepts the named workspace; it does not grant
   authorization to bypass guards or expand the brief.
4. The reviewer reads only: no edits, installs, commits, pushes, merges,
   credential changes or GitHub writes. A missing tool, denied action or a
   material decision returns `QUESTION: <missing information and options>`.
   Do not retry denials with `--force` or `--yolo`. Monitor the checkout for
   unexpected changes; stop and report any unauthorized mutation.
5. Review actual changed behavior and contract compliance. Report high-confidence
   findings with severity, `path:line`, evidence and a concrete fix. Separate
   existing issues from regressions; avoid cosmetic findings.
6. Spot-check findings, fix confirmed issues in the parent session and run the
   appropriate gates. Request a focused recheck for substantive fixes. Record
   what was checked and any limitations; a model's assurance is not a gate.
