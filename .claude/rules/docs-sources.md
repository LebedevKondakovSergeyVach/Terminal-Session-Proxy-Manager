---
paths:
  - "README*.md"
  - "docs/**"
  - "CONTRIBUTING.md"
  - "CHANGELOG*.md"
  - "assets/**"
---

# User documentation — also the website's source

`website/scripts/sync-docs.mjs` generates the site from these files, page for
page, listed in `website/scripts/docs-manifest.mjs`. The same files are read on
GitHub. Full contract: `website/AGENTS.md`, rules 1-5.

## Keep them in lockstep

A change to commands or configuration is incomplete until these agree:
`README.md` **and** `README.ru.md`; the relevant `docs/X.md` **and**
`docs/X.ru.md`; `CHANGELOG.md` **and** `CHANGELOG.ru.md`. `CONTRIBUTING.md` has
no Russian twin — the site falls back to English for it.

## Plain CommonMark

- No JSX, no `import` line. Site-only structure — tabs, cards, steps, asides,
  badges — goes in `<!--site:…-->` comments that GitHub renders as nothing
  (`website/AGENTS.md`, rule 3a). Tab labels and card titles stay real headings.
- A new link target must be in the manifest: linking `docs/NEW.md` without a
  manifest entry fails the build. Fix the manifest, not the check.
- Do not renumber or rename headings casually — their anchors are linked.

## Changelogs

- Keep a Changelog format. Release headings use an em dash:
  `## [2.3.0] — 2026-09-08`. The link-reference footer at the bottom of each file
  needs a line per release, or the heading renders as literal `[x.y.z]`.
- New entries go under `## [Unreleased]`, written for the person upgrading.
- If the open release branch already carries its `## [X.Y.Z]` section (cut but
  not yet merged), ask the maintainer whether a new entry joins that section or
  waits in `[Unreleased]` — CI checks only that both files changed.

## Screenshots

`assets/` images are cached hard by GitHub's image proxy. A changed screenshot
needs a **new filename** and an updated link; overwriting the file shows users
the old image. If the TUI changes materially, say so — the README screenshot
needs retaking.

## Verify

```bash
cd website && npm ci && npm test && npm run build
```

The build fails on a dead internal link. That is the check working.
