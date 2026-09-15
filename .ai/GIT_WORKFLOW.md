# Git Workflow

Branching, review and release rules. Pull requests are checked by
`.github/workflows/branch-policy.yml`, so for them this is not advisory — a pull
request that ignores it fails CI. A direct commit to a release branch is not
checked by that workflow at all, which is why the rules below that it would
enforce have to be kept by hand.

## The model

```
main                    stable; every commit is a released version
 └── release/X.Y.Z      one open release at a time, branched from main
      └── feat/…        optional task branches, branched from the release branch
          fix/…
          docs/…
```

Three rules, and everything else follows:

1. **Work lands on the open release branch.** The maintainer commits to it
   directly; external contributors, and larger changes the maintainer wants
   reviewed, arrive as a task branch merged into it by pull request. Nothing is
   branched from or merged into `main` except a release or hotfix branch.
2. **`main` is advanced only by merging a release or hotfix branch.** Any push to `main`
   runs the release pipeline, so an accidental merge ships a release.
3. **`main` is never committed to directly.**

### Why not merge task branches into main

Merging `release/X.Y.Z` into `main` is the release trigger: CI reads the version
from `Cargo.toml`, builds four targets, creates `vX.Y.Z` and bumps the Homebrew
tap. If task branches went straight to `main`, every merged bugfix would try to
publish, and `main` would hold half-finished versions between releases.

## Branch names

| Prefix | For | Branches from | Merges into |
| :--- | :--- | :--- | :--- |
| `release/X.Y.Z` | Everything going into the next version | `main` | `main` |
| `hotfix/X.Y.Z` | Urgent fix that cannot wait for the open release | `main` | `main` |
| `feat/<slug>` | New capability | release branch | release branch |
| `fix/<slug>` | Bug fix | release branch | release branch |
| `docs/<slug>` | Documentation | release branch | release branch |
| `refactor/<slug>` | Behaviour-preserving change | release branch | release branch |
| `test/<slug>` | Tests only | release branch | release branch |
| `perf/<slug>` | Performance | release branch | release branch |
| `ci/<slug>` | Pipelines and tooling | release branch | release branch |
| `chore/<slug>` | Everything else | release branch | release branch |
| `build/<slug>` | Build system, dependencies | release branch | release branch |

`release/` and `hotfix/` must carry a full three-part SemVer version. Anything
else is rejected.

## Everyday work

Before any commit, run the local gate — the same one everywhere in this
repository's docs:

```bash
cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked
```

CI additionally passes `--all-features` to clippy and the tests. `Cargo.toml`
has no `[features]` table, so locally the two invocations check the same code.

If the change touched `README*.md`, `docs/*.md`, `CONTRIBUTING.md`,
`CHANGELOG*.md`, `assets/` or `website/`, also run the website gate:

```bash
cd website && npm ci && npm test && npm run build
```

### Maintainer: commit to the release branch

```bash
git checkout release/X.Y.Z
git pull
# … edit, run both gates …
git commit -m "fix: …"
git push
```

A push to `release/**` runs `ci.yml`, and `website.yml` when the site's sources
changed. It does **not** run `branch-policy.yml`, which triggers only on pull
requests — so nothing checks the changelog rule below for a direct commit. Keep
it by hand.

### Contributors, and changes that want review: a task branch

Starting a task:

```bash
git checkout release/X.Y.Z
git pull
git checkout -b fix/dashboard-empty-list
```

Finishing it — open a pull request **against the release branch**:

```bash
git push -u origin fix/dashboard-empty-list
gh pr create --base release/X.Y.Z --fill
```

## Opening a release

When the current release ships, branch the next one from `main`:

```bash
git checkout main && git pull
git checkout -b release/X.Y.Z
```

The branch collects work under `## [Unreleased]` in both changelogs until the
release is cut. Pick the number now, because it is in the branch name.

### Choosing the number

SemVer, judged from the user's side rather than the code's:

- **Patch** — a fix that changes no documented behaviour.
- **Minor** — a new command, flag or config field; a behavioural fix people
  might have scripted around.
- **Major** — a removed or renamed command or flag, a changed exit code, a
  config change that needs manual migration.

Exit codes and flag names are interface. Changing one is not a patch.

## Cutting and shipping a release

Cutting is one commit on the release branch (`chore: release X.Y.Z`):

1. `version = "X.Y.Z"` in `Cargo.toml`, then `cargo check` to sync `Cargo.lock`.
2. In **both** `CHANGELOG.md` and `CHANGELOG.ru.md`, rename `## [Unreleased]` to
   `## [X.Y.Z] — YYYY-MM-DD` and open a fresh, empty `## [Unreleased]` above it.
3. Update the link-reference footer of both changelogs: `[Unreleased]` compares
   against `vX.Y.Z`, and a new `[X.Y.Z]` line compares the previous tag with
   `vX.Y.Z`. A heading without a reference renders as literal bracketed text.
4. Run both gates. The changelogs are site pages, so the website gate is not
   optional for a release.

The step-by-step procedure is the release-manager skill,
[`.claude/skills/release-manager/SKILL.md`](../.claude/skills/release-manager/SKILL.md).

Then open a pull request from the release branch to `main`. `branch-policy.yml`'s
release-readiness job checks that:

- the version in the branch name equals the `Cargo.toml` version,
- `vX.Y.Z` is not already tagged,
- `CHANGELOG.md` has a `## [X.Y.Z]` section.

It does not look at `CHANGELOG.ru.md`, and neither does `release.yml` — the
Russian changelog is checked only by you.

Merging it runs `release.yml`: `resolve` sees `main` at a version with no tag;
`verify` re-runs fmt, clippy and tests and checks `CHANGELOG.md` for the
section; `build` produces macOS and Linux binaries for x86_64 and arm64 with
SHA-256 checksums; and only then `publish` creates and pushes the annotated tag,
creates the GitHub release with those files, and bumps the Homebrew tap.

**Nothing is published until that merge.** Pushing a `v*` tag by hand also
starts `release.yml` — `resolve` then requires the tag to match `Cargo.toml` and
`publish` skips creating the tag — but that is for a re-run or an out-of-band
release by the maintainer, not part of everyday work.

After the release, delete the release branch and open the next one from `main`.

### If the release job fails

**Before the tag.** `resolve`, `verify` and `build` all run before `publish`
creates the tag, so a failure in any of them leaves no tag and nothing
published. Fix the cause and run the release again: re-run the workflow for a
transient failure, or, for a code fix, get it onto `main` the only way anything
reaches `main` — merged from a release or hotfix branch. Either way `resolve`
still sees no tag and releases.

**After the tag.** If `publish` fails once `Create tag` has pushed `vX.Y.Z`, a
new run from `main` will not help: `resolve` finds `refs/tags/vX.Y.Z`, sets
`should_release=false`, and every later job is skipped. The tag push itself
started no run either — the header of `release.yml` notes that a tag pushed
with `GITHUB_TOKEN` does not trigger workflows. A run whose ref *is* the tag
(`workflow_dispatch` against `vX.Y.Z`) takes the tag path in `resolve`: it
checks the tag against `Cargo.toml`, sets `should_release=true`, and `publish`
skips `Create tag`. That is the maintainer's call. No step checks for an
existing GitHub release or tap commit, so see what the failed run already
published before starting another.

Do not reuse a tag that already has published artifacts — people may have
downloaded them.

## Rules for agents

- **Never push to `main`.** Not a commit, not a merge, not a tag.
- **Never create a tag.** Tagging is CI's job. Creating one by hand publishes a
  release outside the review flow.
- **Never bump the version outside a release branch,** and never as a side
  effect of another task. The bump belongs to the release cut; a bump anywhere
  else means the next merge to `main` publishes an unintended release.
- **Check where you are before committing** — `git branch --show-current`. If
  it is `main`, stop. Working for the maintainer, commit on the open
  `release/X.Y.Z`; use a task branch and a pull request when the maintainer asks
  for review or when you are contributing from outside.
- **On a task branch: one task, one branch, one pull request.** Committing
  directly: one task per commit. Either way, do not fold unrelated fixes in
  because they were noticed along the way.
- **A direct commit is not checked by `branch-policy.yml`.** Apply the
  changelog rule below yourself.
- Ask before deleting or force-updating any branch.

### Adding a changelog entry

Every change that touches `src/`, `locales/`, `shell/`, `configs/` or
`Cargo.toml` must add a `CHANGELOG.md` entry under `## [Unreleased]`, and the
matching Russian entry in `CHANGELOG.ru.md` — both changelogs are published, so
an entry in one language ships a page that is out of date in the other.
Docs-only and CI-only changes are exempt.

On a pull request into `release/*`, the `changelog` job enforces both: it fails
when those paths changed without `CHANGELOG.md`, and when `CHANGELOG.md` changed
without `CHANGELOG.ru.md`. A direct commit gets no such check, so the rule holds
there by discipline alone.

`shell/` and `configs/` count as user-facing: the shell function is the
documented integration point, and `config.default.json` ships the defaults
people meet on first run.

Write for the person upgrading, not the person reviewing the diff: say what
changed for them and, for a fix, what went wrong before.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`,
`docs:`, `chore:`, `refactor:`, `test:`, `perf:`, `ci:`, `build:`.

The subject says what changed and why it matters, not which files were touched.
The body carries the reasoning — what was broken, what a user would have seen,
why this fix over another.

## What CI runs, and when

| Workflow | Trigger | Checks |
| :--- | :--- | :--- |
| `ci.yml` | Every pull request; pushes to `main`, `release/**`, `hotfix/**`; weekly; `workflow_dispatch` | fmt, clippy, docs, tests on Linux and macOS, MSRV, `cargo audit` |
| `branch-policy.yml` | Every pull request (only) | Branch naming, head/base pairing; changelog entry on PRs into `release/*`; release readiness on `release/*` → `main` |
| `release.yml` | Any push to `main`; a pushed `v*` tag; `workflow_dispatch` | Decides whether to release, re-verifies, builds four targets, tags, publishes, bumps Homebrew |
| `website.yml` | Pull requests, and pushes to `main`/`release/**`/`hotfix/**`, touching `website/`, `docs/`, `assets/`, the READMEs, `CONTRIBUTING.md` or `CHANGELOG*.md`; `workflow_dispatch` | Generator tests, full site build, internal-link validation, a check that **no generated page is tracked**, `npm audit` |
| `pages.yml` | Pushes to **`main`** touching those same paths; `workflow_dispatch` | Rebuilds and publishes the site to GitHub Pages |

The "no generated page is tracked" step is the one that catches a committed
`.mdx`: everything under `website/src/content/docs/` is generated and ignored,
bar the two home pages and the two 404 pages.

Warnings are failures. `ci.yml` sets `RUSTFLAGS: -D warnings` for every job;
`release.yml` does not set `RUSTFLAGS`, and instead passes `-D warnings` to
clippy directly.

A manual `workflow_dispatch` runs against whichever ref it is started on, so
treat one as a deliberate act. For `release.yml` a dispatch on a tag releases
that tag, as described above.

`pages.yml`'s push trigger is limited to `main`, but its `workflow_dispatch` is
not — a dispatch is the one way to publish from another ref. Pointing it at a
release or task branch publishes unreviewed documentation to the live URL: the
`build` job (read-only token) runs that branch's `npm ci` lifecycle scripts and
generator, and the `deploy` job, which alone holds `pages: write` and
`id-token: write`, publishes whatever they produced. If you need
to preview the site, build it locally
(`cd website && npm run build && npm run preview`).
