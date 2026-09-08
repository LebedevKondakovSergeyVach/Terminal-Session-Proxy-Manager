# Website rebuild — design

Date: 2026-08-22
Scope: `website/` only. Deployment wiring, root docs and CI are deliberately out of scope.

## Problem

`website/` is a scaffold that cannot be deployed correctly and already emits
broken links.

Evidence from the committed build output:

| Symptom | Location |
| :--- | :--- |
| `href="installation"` resolves to `/overview/installation` | `dist/overview/index.html` |
| `href="CHANGELOG.md"`, `CONTRIBUTING.md`, `LICENSE`, `SECURITY.md` point at repo files that do not exist on the site | `dist/overview/index.html` |
| Template leftovers ship to production and are indexed by Pagefind | `dist/{guides,reference}/example/` |

Root causes:

1. `astro.config.mjs` sets neither `site` nor `base`. The site is a GitHub
   Pages project page, served from `/Terminal-Session-Proxy-Manager/`.
2. `scripts/sync-docs.js` transforms Markdown with regular expressions. Its
   link table produces bare relative paths, which resolve against the current
   directory rather than the site root.
3. Generated pages are not ignored by Git, so they drift from their sources.
4. `custom.css` fights `starlight-theme-md3` through fifteen `!important`
   declarations.
5. `ThemeSelect.astro` disables `::view-transition-*(root)` globally, which
   also disables the page transitions `astro-vtbot` provides.

## Decisions

| Decision | Choice |
| :--- | :--- |
| Hosting | GitHub Pages project page: `site` = `https://lebedevkondakovsergeyvach.github.io`, `base` = `/Terminal-Session-Proxy-Manager` |
| Source of truth for docs | Root `README*.md` and `docs/*.md` stay canonical; the website generates from them |
| Theme | Drop `starlight-theme-md3`; a thin custom layer over stock Starlight |
| View transitions | Keep `astro-vtbot`; fix the conflict rather than removing the feature |
| Generated content | Ignored by Git, produced at build time |

`site` and `base` are set now even though deployment is deferred. Their absence
is what produced the broken-link class above; retrofitting them later means
finding every one of those links again.

## Architecture

### Shared configuration

`website/site.config.mjs` exports `SITE` and `BASE`. Both `astro.config.mjs`
and `scripts/sync-docs.js` import it, so the base path has exactly one
definition.

### Content pipeline

`scripts/sync-docs.js` is rewritten around a declarative manifest. Each entry
names a source file, a destination slug, a locale, per-locale title and
description, and the cleanup rules that apply to it.

Cleanup rules replace the current pattern-guessing. The seven sources have two
shapes:

- `README.md` and `README.ru.md`: a language-switcher HTML block, then `# H1`,
  then a banner image, then five badge lines, then prose.
- `docs/*.md`, `CONTRIBUTING.md`, `CHANGELOG*.md`: `# H1`, then prose.

The manifest sets `stripPreamble`, `stripBadges` and `stripHeroImage` per
source. The script asserts the expected shape before transforming and throws
when it does not match. A malformed source fails the build instead of silently
producing a damaged page.

This deliberately avoids adding markers to the root documents, which are out of
scope. The trade-off is that editing a README header breaks the site build.
Loud failure is preferred to silent corruption.

Link handling: every Markdown link is resolved through an explicit table.

- Links to a synced document become base-prefixed absolute site paths.
- Links to repository files that have no page — `LICENSE`, `SECURITY.md`,
  `configs/config.default.json` — become absolute GitHub URLs.
- Anything unresolved throws.

Frontmatter is serialised with proper YAML escaping rather than string
concatenation, so a title containing `:` cannot break the build.

Output is written to `src/content/docs/` and ignored by Git. Drift between
source and site becomes structurally impossible because the generated files do
not exist outside a build.

### Metadata

Starlight's `utils/head.ts` already emits canonical, `og:title`, `og:type`,
`og:url`, `og:locale`, `og:description`, `og:site_name`, `twitter:card`,
`hreflang` alternates and the sitemap link. It does not emit `og:image`.

Therefore no `Head.astro` override. Starlight's own reference calls overriding
that component a last resort. The missing tags are added through the `head`
config option instead.

`@astrojs/sitemap` is not added as a dependency: Starlight wires it up itself
once `site` is set.

### Localisation

`CONTRIBUTING.ru.md` does not exist. Starlight's fallback already renders the
English page at `/ru/contributing/`, so this is not a broken link — but a
Russian reader silently receives English. The sidebar entry carries a
per-locale `badge` marking it English-only, which is Starlight's documented
mechanism for this.

### Styling

`custom.css` is rewritten against Starlight's documented custom properties
(`--sl-color-accent*`, `--sl-font`, `--sl-font-mono`) with no `!important`.
Fonts are self-hosted through `@fontsource-variable/*` rather than a blocking
`@import` from a third-party CDN.

`ThemeSelect.astro` keeps its circular reveal, but the
`::view-transition-*(root)` overrides are scoped behind a
`:root[data-theme-transition]` attribute set only while the theme changes. Page
transitions and the theme reveal then coexist.

### Plugins

Added: `starlight-links-validator` (a broken link fails the build),
`starlight-image-zoom` (TUI screenshots), `starlight-llms-txt` (`llms.txt`,
consistent with this repository's agent-first posture).

Removed: `starlight-theme-md3`.

Rejected: `astro-og-canvas` — it pulls `canvaskit-wasm` to generate per-page
images, where a single site-wide `og:image` is sufficient.

## Out of scope

Deployment workflow, the CI gate, the landing page redesign, root `AGENTS.md`,
`docs/` and `.agents/skills/`. The landing page is touched only so its hero
links survive `base`.

## Verification

`npm run build` must succeed with the link validator enabled, and
`npm run preview` must serve a site rooted at `/Terminal-Session-Proxy-Manager/`
with working navigation in both locales.
