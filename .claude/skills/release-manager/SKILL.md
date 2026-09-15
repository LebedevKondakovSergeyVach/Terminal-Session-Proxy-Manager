---
name: release-manager
description: Use when the user asks to cut, prepare or ship a release, open the next release branch, bump the crate version, or finalise the changelogs for a version.
---

# release-manager

Two separate moments. Ask which one the user means if it is not obvious.
Background and CI behaviour: `.ai/GIT_WORKFLOW.md`, "Opening a release" and
"Cutting and shipping a release".

## A. Opening the next release branch

Only after the previous release has merged into `main`.

1. Ask for the version if it was not given. SemVer from the user's side: a new
   command or flag is minor; a renamed flag or changed exit code is major.
2. `git checkout main && git pull`, then `git checkout -b release/X.Y.Z`.
3. Do not bump `Cargo.toml` yet. Work collects under `## [Unreleased]`.
4. `git push -u origin release/X.Y.Z` — pushing a release branch is allowed;
   pushing `main` is not.

## B. Cutting the release on the open branch

1. `git branch --show-current` must print `release/X.Y.Z`. Stop otherwise.
2. `version = "X.Y.Z"` in `Cargo.toml`; `cargo check` to sync `Cargo.lock`.
3. In **both** `CHANGELOG.md` and `CHANGELOG.ru.md`: rename `## [Unreleased]` to
   `## [X.Y.Z] — YYYY-MM-DD` (em dash, as every existing heading) and add a new
   empty `## [Unreleased]` above it. If `[Unreleased]` was empty, ask — there may
   be nothing to release.
4. Update the link-reference footer at the bottom of **both** files:

   ```
   [Unreleased]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/vX.Y.Z...HEAD
   [X.Y.Z]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v<previous>...vX.Y.Z
   ```

   A heading without its reference renders as literal `[X.Y.Z]` on GitHub and
   on the site; 2.2.1 and 2.2.2 shipped that way.
5. Run the `verify-project` skill — both parts. The changelogs are site pages.
6. Commit `chore: release X.Y.Z` — no attribution lines — and push the release
   branch.

## C. Shipping

Merging `release/X.Y.Z` into `main` runs `release.yml`: it builds four targets,
then tags `vX.Y.Z`, publishes the GitHub release, bumps the Homebrew tap, and
`pages.yml` redeploys the site. **Say this to the user and get an explicit yes
before opening the pull request.**

```bash
gh auth status            # gh must be logged in; if not, ask the user to run `gh auth login`
gh pr create --base main --head release/X.Y.Z --title "Release X.Y.Z" --body-file <file>
```

The body follows `.github/PULL_REQUEST_TEMPLATE.md` and carries no "Generated
with" line. Never merge it yourself, never push `main`, never create a tag —
`.githooks/pre-push` refuses pushing `main` or any tag.

## Common mistakes

| Mistake | Consequence |
| :--- | :--- |
| Bumping the version on a task branch or while opening the release | the next merge to `main` publishes early |
| Updating only `CHANGELOG.md` | `CHANGELOG.ru.md` is a published page; CI does not check it at release time |
| Forgetting the footer references | headings render as bracketed text |
| Skipping the website build | a dead changelog link surfaces only after the merge, in `pages.yml` |
