# Documentation website

The [Astro Starlight](https://starlight.astro.build/) documentation site for
Terminal Session Proxy Manager, deployed as a GitHub Pages project page at
`/Terminal-Session-Proxy-Manager/`, in English and Russian.

`.github/workflows/website.yml` gates every pull request that touches the site
or its sources; `.github/workflows/pages.yml` publishes from `main`.

## Commands

Run from this directory.

| Command | Action |
| :--- | :--- |
| `npm install` | Install dependencies |
| `npm run sync` | Regenerate content from the repository's Markdown |
| `npm run dev` | Dev server on `localhost:4321` (syncs first) |
| `npm run build` | Production build into `dist/` (syncs first, validates links) |
| `npm run preview` | Serve the built site |
| `npm test` | Content generator and dependency compatibility tests |

## How content works

Documentation is **not** written here. The canonical sources are the
repository's own Markdown — `README.md`, `README.ru.md`, `docs/*.md`,
`CONTRIBUTING.md` and `CHANGELOG*.md` — so they stay readable on GitHub.

`scripts/sync-docs.mjs` generates the site's pages from them at build time:

- `scripts/docs-manifest.mjs` declares each page's source, slug, locale, title
  and cleanup rules.
- `scripts/lib/cleanup.mjs` strips GitHub chrome — language switchers, the
  banner, status badges — asserting each rule matched.
- `scripts/lib/components.mjs` expands the `<!--site:…-->` comments into
  Starlight components. The sources stay plain CommonMark so they read
  correctly on GitHub; the site-only structure lives in comments GitHub
  ignores.
- `scripts/lib/links.mjs` rewrites every link to a base-prefixed site path or
  an absolute GitHub URL, and throws on anything unrecognised.
- `scripts/lib/frontmatter.mjs` serialises frontmatter as valid YAML.

Generated pages are Git-ignored — `.md` and `.mdx` alike — so the site cannot
drift from its sources. The only hand-authored pages are the home pages,
`index.mdx` and `ru/index.mdx`, and the 404 pages, `404.md` and `ru/404.md`.

`astro.config.mjs` pins `markdown.processor` to `unified()` rather than
Astro 7's default Sätteri processor: `starlight-image-zoom` doesn't yet
support Sätteri.

## Dependency compatibility

The `starlight-llms-txt` dependency uses only `micromatch.isMatch`, which
delegates to Picomatch. A scoped npm override supplies Picomatch 4.0.7 directly
to remove the unpatched `braces` dependency
([GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm)).
The matcher tests cover homepage promotion, empty lists and page selectors.
Recheck the plugin's matcher API and generated `llms*.txt` files when updating
it; compatibility with Micromatch is limited to the `isMatch` API used here.

## Structure

```text
website/
├── site.config.mjs        # site + base, imported by the config and the generator
├── astro.config.mjs       # Starlight config: locales, sidebar, plugins
├── scripts/               # content generator and its tests
├── icons/                 # 1024px sources of the favicon and the logo
├── public/                # served at the site root (favicon.png, apple-touch-icon.png, og.jpg)
└── src/
    ├── assets/            # images processed by Astro, including logo.jpg
    ├── components/        # overrides: ThemeSelect, Search, SiteTitle, MarkdownContent
    ├── content/docs/      # index.mdx and 404.md (both locales) are hand-written; the rest is generated
    ├── content/i18n/      # UI strings: pagefind.* and the site's own tspm.*
    ├── scripts/           # diagram viewer gestures, zoom and focus handling
    └── styles/            # layout, responsive rules and Mermaid theme
```

## For agents

Read [`AGENTS.md`](AGENTS.md) before changing anything here.
