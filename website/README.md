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
| `npm test` | Unit tests for the content generator |

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
drift from its sources. The only hand-authored pages are `index.mdx` and
`ru/index.mdx`.

`astro.config.mjs` pins `markdown.processor` to `unified()` rather than
Astro 7's default Sätteri processor: `starlight-image-zoom` doesn't yet
support Sätteri.

## Structure

```text
website/
├── site.config.mjs        # site + base, imported by the config and the generator
├── astro.config.mjs       # Starlight config: locales, sidebar, plugins
├── scripts/               # content generator and its tests
├── public/                # served at the site root (favicon, og.jpg)
└── src/
    ├── assets/            # images processed by Astro
    ├── components/        # Starlight component overrides
    ├── content/docs/      # index.mdx is hand-written; the rest is generated
    └── styles/custom.css  # theme layer over stock Starlight
```

## For agents

Read [`AGENTS.md`](AGENTS.md) before changing anything here.
