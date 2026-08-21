# Git Workflow

Branching, review and release rules. Enforced by
`.github/workflows/branch-policy.yml`, so this is not advisory — a pull request
that ignores it fails CI.

## The model

```
main                    stable; every commit is a released version
 └── release/X.Y.Z      one open release at a time, branched from main
      └── feat/…        task branches, branched from the release branch
          fix/…
          docs/…
```

Three rules, and everything else follows:

1. **Task branches come from the release branch and merge back into it.**
   Never from or into `main`.
2. **`main` is advanced only by merging a release branch.** That merge is what
   creates the tag and publishes, so an accidental merge ships a release.
3. **`main` is never committed to directly.**

### Why not merge task branches into main

Merging `release/X.Y.Z` into `main` is the release trigger: CI reads the version
from `Cargo.toml`, creates `vX.Y.Z`, builds four targets and bumps the Homebrew
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

Starting a task:

```bash
git checkout release/2.3.0
git pull
git checkout -b fix/dashboard-empty-list
```

Finishing it — open a pull request **against the release branch**:

```bash
git push -u origin fix/dashboard-empty-list
gh pr create --base release/2.3.0 --fill
```

Before pushing, run what CI runs:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
```

## Opening a release

When the current release ships, branch the next one from `main`:

```bash
git checkout main && git pull
git checkout -b release/2.3.0
```

Then set the version immediately, so every later check has something to compare
against:

1. `version = "2.3.0"` in `Cargo.toml`
2. `cargo check` to sync `Cargo.lock`
3. A `## [2.3.0]` section in `CHANGELOG.md`

The branch name and the `Cargo.toml` version must agree; CI compares them.

### Choosing the number

SemVer, judged from the user's side rather than the code's:

- **Patch** — a fix that changes no documented behaviour.
- **Minor** — a new command, flag or config field; a behavioural fix people
  might have scripted around.
- **Major** — a removed or renamed command or flag, a changed exit code, a
  config change that needs manual migration.

Exit codes and flag names are interface. Changing one is not a patch.

## Shipping

Open a pull request from the release branch to `main`. `branch-policy.yml`
checks that:

- the branch name matches the `Cargo.toml` version,
- `vX.Y.Z` is not already tagged,
- `CHANGELOG.md` has a `## [X.Y.Z]` section.

Merging it runs `release.yml`, which re-verifies fmt, clippy and tests, creates
the annotated tag, builds macOS and Linux binaries for x86_64 and arm64,
publishes them with SHA-256 checksums, and bumps the Homebrew tap.

**Nothing is published until that merge.** Pushing a tag by hand also works and
takes the same path, for a re-run or an out-of-band release.

After the release, delete the release branch and open the next one from `main`.

### If the release job fails

The tag is created before the binaries are built, so a build failure leaves a
tag with no release. Fix forward:

```bash
git push --delete origin v2.3.0    # then re-merge, or push the tag again
```

Do not reuse a tag that already has published artifacts — people may have
downloaded them.

## Rules for agents

- **Never push to `main`.** Not a commit, not a merge, not a tag.
- **Never create a tag.** Tagging is CI's job. Creating one by hand publishes a
  release outside the review flow.
- **Never bump the version outside a release branch,** and never as a side
  effect of another task. A version bump on a task branch means the next merge
  to `main` publishes an unintended release.
- **Check where you are before committing** — `git branch --show-current`. If
  it is `main`, stop and branch.
- **One task, one branch, one pull request.** Do not fold unrelated fixes into a
  branch because they were noticed along the way; open a second branch.
- Ask before deleting or force-updating any branch.

### Adding a changelog entry

Every pull request that touches `src/`, `locales/` or `Cargo.toml` must add a
`CHANGELOG.md` entry under `## [Unreleased]`; CI enforces it. Docs-only and
CI-only changes are exempt.

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
| `ci.yml` | Every pull request; pushes to `main`, `release/**`, `hotfix/**`; weekly | fmt, clippy, docs, tests on Linux and macOS, MSRV, `cargo audit` |
| `branch-policy.yml` | Every pull request | Branch naming, head/base pairing, changelog entry, release readiness |
| `release.yml` | Merge into `main`; a pushed `v*` tag | Re-verifies, tags, builds four targets, publishes, bumps Homebrew |

Warnings are failures: CI sets `RUSTFLAGS: -D warnings`.
