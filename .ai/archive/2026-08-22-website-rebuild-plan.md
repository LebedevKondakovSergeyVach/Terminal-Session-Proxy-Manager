# Website Rebuild Implementation Plan

> [!WARNING]
> **Archived.** This document describes the project as it was on the date
> below, and parts of it are no longer true. It is kept for its reasoning, not
> as guidance — see [`README.md`](README.md) in this directory. The authority
> for current behaviour is `AGENTS.md`, `website/AGENTS.md` and `.ai/`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn `website/` from a scaffold that emits broken links into a correct, deployable Astro Starlight documentation site for a GitHub Pages project page.

**Architecture:** One shared module defines `site` and `base` for both the Astro config and the docs generator. The generator is rebuilt as a declarative manifest plus three small, unit-tested modules (shape-based cleanup, link resolution, frontmatter serialisation) that fail loudly instead of silently producing dead links. Generated pages are Git-ignored, so source and site cannot drift. Styling drops the Material Design 3 theme for a thin layer over stock Starlight.

**Tech Stack:** Astro 7.2.4, `@astrojs/starlight` 0.41.7, Node 22+ (`node:test`, no test dependencies), `astro-vtbot`, `starlight-links-validator`, `starlight-image-zoom`, `starlight-llms-txt`, `@fontsource-variable/*`.

**Spec:** [`2026-08-22-website-rebuild-design.md`](2026-08-22-website-rebuild-design.md)

## Global Constraints

- All work stays inside `website/`, except the new root file `.mcp.json`. Do not modify `README.md`, `README.ru.md`, `docs/*.md`, `CONTRIBUTING.md`, `CHANGELOG*.md`, root `AGENTS.md`, `.github/`, or `.agents/`.
- `SITE` = `https://lebedevkondakovsergeyvach.github.io`
- `BASE` = `/Terminal-Session-Proxy-Manager`
- Repository URL for links to non-page files: `https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/blob/main/`
- Run every command from `website/`.
- All new script files use the `.mjs` extension. `website/package.json` already sets `"type": "module"`.
- The generator must throw on any input it does not recognise. Silent fallbacks are the defect this plan exists to remove.
- Content is bilingual: every page exists for `en` (root) and `ru`, except `contributing`, which is English-only by design.
- Do not commit unless explicitly asked. Steps that say "commit" are for the user to trigger.

---

### Task 1: Shared site configuration and Astro config baseline

Establishes `site`/`base` and removes the Material Design 3 theme. Everything downstream depends on `BASE` being importable.

**Files:**
- Create: `website/site.config.mjs`
- Modify: `website/astro.config.mjs`
- Modify: `website/package.json`

**Interfaces:**
- Consumes: nothing.
- Produces: `website/site.config.mjs` exporting `SITE: string`, `BASE: string`, `REPO_BLOB_URL: string`. Imported by `astro.config.mjs` and by every generator module.

- [ ] **Step 1: Create the shared config**

```js
// website/site.config.mjs
// Single definition of the deployment origin. Both the Astro config and the
// docs generator import this: the generator has to emit base-prefixed links
// itself, because Starlight only prefixes sidebar and site-title links.

/** Origin the site is served from. */
export const SITE = 'https://lebedevkondakovsergeyvach.github.io';

/** Sub-path of the GitHub Pages project page. No trailing slash. */
export const BASE = '/Terminal-Session-Proxy-Manager';

/** Prefix for links to repository files that have no page on the site. */
export const REPO_BLOB_URL =
	'https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/blob/main/';
```

- [ ] **Step 2: Remove the Material Design 3 theme dependency**

Run:
```bash
npm uninstall starlight-theme-md3
```

- [ ] **Step 3: Rewrite the Astro config**

Replace the whole of `website/astro.config.mjs`:

```js
// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { viewTransitions } from 'astro-vtbot/starlight-view-transitions';

import { SITE, BASE } from './site.config.mjs';

// https://astro.build/config
export default defineConfig({
	site: SITE,
	base: BASE,
	// Pinned explicitly rather than left to the default: the generator emits
	// absolute links with a trailing slash, and dev has to resolve them the
	// same way the static build on GitHub Pages does.
	trailingSlash: 'always',
	build: { format: 'directory' },
	integrations: [
		starlight({
			title: 'Terminal Session Proxy Manager',
			plugins: [viewTransitions()],
			defaultLocale: 'root',
			locales: {
				root: { label: 'EN', lang: 'en' },
				ru: { label: 'RU', lang: 'ru' },
			},
			components: {
				ThemeSelect: './src/components/ThemeSelect.astro',
			},
			customCss: ['./src/styles/custom.css'],
			social: [
				{
					icon: 'github',
					label: 'GitHub',
					href: 'https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager',
				},
			],
			sidebar: [
				{
					label: 'Start Here',
					translations: { ru: 'Начало работы' },
					items: [
						{ label: 'Overview', translations: { ru: 'Обзор' }, slug: 'overview' },
						{ label: 'Installation', translations: { ru: 'Установка' }, slug: 'installation' },
						{ label: 'Changelog', translations: { ru: 'История изменений' }, slug: 'changelog' },
					],
				},
				{
					label: 'Guides',
					translations: { ru: 'Руководства' },
					items: [
						{ label: 'Configuration', translations: { ru: 'Конфигурация' }, slug: 'configuration' },
						{ label: 'Usage', translations: { ru: 'Использование' }, slug: 'usage' },
						{
							label: 'Shell Integration',
							translations: { ru: 'Интеграция с Shell' },
							slug: 'shell-integration',
						},
					],
				},
				{
					label: 'Project',
					translations: { ru: 'Проект' },
					items: [
						{
							label: 'Contributing',
							translations: { ru: 'Разработка' },
							slug: 'contributing',
							// No CONTRIBUTING.ru.md exists. Starlight falls back to the
							// English page rather than 404ing, so say so instead of
							// silently serving English under a Russian label.
							badge: { text: { en: 'EN', ru: 'на английском' }, variant: 'note' },
						},
					],
				},
			],
		}),
	],
});
```

- [ ] **Step 4: Verify the build produces base-prefixed output**

Run:
```bash
npm run build
grep -o 'href="/Terminal-Session-Proxy-Manager/[^"]*"' dist/overview/index.html | sort -u | head
```
Expected: sidebar hrefs all begin with `/Terminal-Session-Proxy-Manager/`. The build itself may still warn about content; that is fixed in Task 2.

- [ ] **Step 5: Commit**

```bash
git add website/site.config.mjs website/astro.config.mjs website/package.json website/package-lock.json
git commit -m "build(website): set site and base, drop starlight-theme-md3"
```

---

### Task 2: Docs manifest and shape-based cleanup

Replaces pattern-guessing with a declarative manifest and assertions that fail the build when a source document changes shape.

**Files:**
- Create: `website/scripts/docs-manifest.mjs`
- Create: `website/scripts/lib/frontmatter.mjs`
- Create: `website/scripts/lib/cleanup.mjs`
- Create: `website/scripts/lib/cleanup.test.mjs`
- Create: `website/scripts/lib/frontmatter.test.mjs`
- Modify: `website/package.json` (add the `test` script)

**Interfaces:**
- Consumes: `SITE`, `BASE`, `REPO_BLOB_URL` from Task 1.
- Produces:
  - `docs-manifest.mjs` exports `PAGES: PageEntry[]` where
    `PageEntry = { source: string, slug: string, locale: 'en' | 'ru', dest: string, title: string, description: string, stripPreamble: boolean, stripBadges: boolean, stripHeroImage: boolean }`.
  - `cleanup.mjs` exports `extractTitle(markdown: string): { title: string, body: string }` and `applyCleanup(markdown: string, entry: PageEntry): string`.
  - `frontmatter.mjs` exports `yamlString(value: string): string` and `buildFrontmatter(fields: Record<string, string>): string`.

- [ ] **Step 1: Write the failing tests for frontmatter serialisation**

```js
// website/scripts/lib/frontmatter.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';

import { yamlString, buildFrontmatter } from './frontmatter.mjs';

test('a plain string is single-quoted', () => {
	assert.equal(yamlString('Installation'), "'Installation'");
});

test('a colon in a title cannot break the document', () => {
	assert.equal(yamlString('Setup: macOS'), "'Setup: macOS'");
});

test('a single quote is doubled rather than escaped with a backslash', () => {
	assert.equal(yamlString("Don't"), "'Don''t'");
});

test('buildFrontmatter emits a delimited block with a trailing blank line', () => {
	assert.equal(
		buildFrontmatter({ title: 'A', description: 'B' }),
		"---\ntitle: 'A'\ndescription: 'B'\n---\n\n"
	);
});
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `node --test scripts/lib/frontmatter.test.mjs`
Expected: FAIL — `Cannot find module './frontmatter.mjs'`

- [ ] **Step 3: Implement frontmatter serialisation**

```js
// website/scripts/lib/frontmatter.mjs

/**
 * Serialise a string as a YAML single-quoted scalar.
 *
 * The previous generator concatenated titles straight into the frontmatter, so
 * any title containing a colon produced an unparsable document.
 */
export function yamlString(value) {
	return `'${String(value).replace(/'/g, "''")}'`;
}

/** Build a frontmatter block from an ordered map of scalar fields. */
export function buildFrontmatter(fields) {
	const body = Object.entries(fields)
		.map(([key, value]) => `${key}: ${yamlString(value)}`)
		.join('\n');
	return `---\n${body}\n---\n\n`;
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `node --test scripts/lib/frontmatter.test.mjs`
Expected: PASS, 4 tests.

- [ ] **Step 5: Write the failing tests for cleanup**

```js
// website/scripts/lib/cleanup.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';

import { extractTitle, applyCleanup } from './cleanup.mjs';

const README = [
	'<p align="center">',
	'  🇬🇧 <b>English</b> | 🇷🇺 <a href="README.ru.md">Русский</a>',
	'</p>',
	'',
	'# ⚡ Terminal Session Proxy Manager',
	'',
	'![Project Banner](assets/banner_new.jpg)',
	'',
	'[![CI](https://github.com/o/r/actions/workflows/ci.yml/badge.svg)](https://github.com/o/r/actions)',
	'[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)',
	'![Views](https://komarev.com/ghpvc/?username=x&label=views)',
	'',
	'Manage proxy profiles.',
	'',
].join('\n');

const entry = {
	source: 'README.md',
	stripPreamble: true,
	stripBadges: true,
	stripHeroImage: true,
};

test('extractTitle returns the first H1 and removes it from the body', () => {
	const { title, body } = extractTitle('# ⚡ Title\n\nProse.\n');
	assert.equal(title, '⚡ Title');
	assert.equal(body.includes('# ⚡ Title'), false);
	assert.equal(body.includes('Prose.'), true);
});

test('extractTitle throws when no H1 appears near the top', () => {
	assert.throws(() => extractTitle('\n\nJust prose, no heading.\n'), /H1/);
});

test('the HTML preamble above the H1 is removed', () => {
	const out = applyCleanup(README, entry);
	assert.equal(out.includes('align="center"'), false);
});

test('the banner image is removed', () => {
	const out = applyCleanup(README, entry);
	assert.equal(out.includes('banner_new.jpg'), false);
});

test('badge lines are removed but prose survives', () => {
	const out = applyCleanup(README, entry);
	assert.equal(out.includes('img.shields.io'), false);
	assert.equal(out.includes('komarev.com'), false);
	assert.equal(out.includes('badge.svg'), false);
	assert.equal(out.includes('Manage proxy profiles.'), true);
});

test('a content image survives while a badge on the same document is removed', () => {
	const mixed = [
		'# T',
		'',
		'[![Rust](https://img.shields.io/badge/rust-1.88-orange.svg)](https://www.rust-lang.org)',
		'',
		'![Dashboard](assets/proxy_dashboard_final.png)',
		'',
		'Prose.',
	].join('\n');
	const out = applyCleanup(mixed, {
		source: 'docs/USAGE.md',
		stripPreamble: false,
		stripBadges: true,
		stripHeroImage: false,
	});
	assert.equal(out.includes('img.shields.io'), false);
	assert.equal(out.includes('proxy_dashboard_final.png'), true);
});

test('stripBadges throws when the flag is set but the document has no badges', () => {
	assert.throws(
		() =>
			applyCleanup('# T\n\nProse only.\n', {
				source: 'README.md',
				stripPreamble: false,
				stripBadges: true,
				stripHeroImage: false,
			}),
		/badge/i
	);
});
```

- [ ] **Step 6: Run the tests to verify they fail**

Run: `node --test scripts/lib/cleanup.test.mjs`
Expected: FAIL — `Cannot find module './cleanup.mjs'`

- [ ] **Step 7: Implement cleanup**

```js
// website/scripts/lib/cleanup.mjs

/**
 * Hosts that only ever serve status badges. A line consisting solely of images
 * from these hosts is chrome for GitHub readers and noise on the site.
 */
const BADGE_PATTERNS = [/img\.shields\.io/, /komarev\.com/, /\/badge\.svg/];

/** How far into a document the H1 is allowed to appear. */
const H1_SEARCH_LINES = 15;

/**
 * Split the first H1 off the document.
 *
 * Starlight renders the title from frontmatter, so leaving the H1 in the body
 * would render it twice.
 */
export function extractTitle(markdown) {
	const lines = markdown.split('\n');
	const index = lines.findIndex((line, i) => i < H1_SEARCH_LINES && /^#\s+\S/.test(line));
	if (index === -1) {
		throw new Error(
			`Expected an H1 within the first ${H1_SEARCH_LINES} lines; the document shape has changed.`
		);
	}
	const title = lines[index].replace(/^#\s+/, '').trim();
	lines.splice(index, 1);
	return { title, body: lines.join('\n') };
}

function isBadgeLine(line) {
	const trimmed = line.trim();
	if (trimmed === '') return false;
	// Must be images or linked images only — no prose on the line.
	if (!/^(\[?!\[[^\]]*\]\([^)]*\)\]?(\([^)]*\))?\s*)+$/.test(trimmed)) return false;
	return BADGE_PATTERNS.some((pattern) => pattern.test(trimmed));
}

function isHeroImageLine(line) {
	return /^!\[[^\]]*\]\(assets\/banner_new\.jpg\)\s*$/.test(line.trim());
}

/**
 * Apply the per-source cleanup rules from the manifest.
 *
 * Each rule asserts that it actually matched. The generator this replaces
 * guessed at these shapes with regular expressions and produced damaged pages
 * when it guessed wrong; here a shape change stops the build instead.
 */
export function applyCleanup(markdown, entry) {
	let lines = markdown.split('\n');

	if (entry.stripPreamble) {
		const h1 = lines.findIndex((line) => /^#\s+\S/.test(line));
		if (h1 === -1) {
			throw new Error(`${entry.source}: stripPreamble is set but the document has no H1.`);
		}
		lines = lines.slice(h1);
	}

	if (entry.stripHeroImage) {
		const hero = lines.findIndex(isHeroImageLine);
		if (hero === -1) {
			throw new Error(
				`${entry.source}: stripHeroImage is set but no banner image line was found.`
			);
		}
		lines.splice(hero, 1);
	}

	if (entry.stripBadges) {
		const kept = lines.filter((line) => !isBadgeLine(line));
		if (kept.length === lines.length) {
			throw new Error(`${entry.source}: stripBadges is set but no badge line was found.`);
		}
		lines = kept;
	}

	return lines.join('\n').replace(/\n{3,}/g, '\n\n').trimStart();
}
```

- [ ] **Step 8: Run the tests to verify they pass**

Run: `node --test scripts/lib/cleanup.test.mjs`
Expected: PASS, 7 tests.

- [ ] **Step 9: Write the manifest**

```js
// website/scripts/docs-manifest.mjs
// The single declaration of what the site is built from. Titles and
// descriptions live here rather than in the sources, because the sources are
// GitHub documents and are out of scope for this work.

/**
 * @typedef {object} PageEntry
 * @property {string}  source         Path relative to the repository root.
 * @property {string}  slug           Site slug, without locale prefix or slashes.
 * @property {'en'|'ru'} locale
 * @property {string}  dest           Path relative to `src/content/docs/`.
 * @property {string}  description    Used for `og:description` and search results.
 * @property {boolean} stripPreamble  Remove everything above the first H1.
 * @property {boolean} stripBadges    Remove status-badge-only lines.
 * @property {boolean} stripHeroImage Remove the repository banner image.
 */

/** @type {PageEntry[]} */
export const PAGES = [
	{
		source: 'README.md',
		slug: 'overview',
		locale: 'en',
		dest: 'overview.md',
		description: 'What Terminal Session Proxy Manager does and why it exists.',
		stripPreamble: true,
		stripBadges: true,
		stripHeroImage: true,
	},
	{
		source: 'README.ru.md',
		slug: 'overview',
		locale: 'ru',
		dest: 'ru/overview.md',
		description: 'Что такое Terminal Session Proxy Manager и зачем он нужен.',
		stripPreamble: true,
		stripBadges: true,
		stripHeroImage: true,
	},
	{
		source: 'docs/INSTALLATION.md',
		slug: 'installation',
		locale: 'en',
		dest: 'installation.md',
		description: 'Installing on macOS and Linux via Homebrew, Cargo or a prebuilt binary.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/INSTALLATION.ru.md',
		slug: 'installation',
		locale: 'ru',
		dest: 'ru/installation.md',
		description: 'Установка на macOS и Linux через Homebrew, Cargo или готовый бинарник.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/CONFIGURATION.md',
		slug: 'configuration',
		locale: 'en',
		dest: 'configuration.md',
		description: 'Profiles, settings and the precedence rules for config file resolution.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/CONFIGURATION.ru.md',
		slug: 'configuration',
		locale: 'ru',
		dest: 'ru/configuration.md',
		description: 'Профили, настройки и правила разрешения путей к конфигурации.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/USAGE.md',
		slug: 'usage',
		locale: 'en',
		dest: 'usage.md',
		description: 'Every subcommand, its flags and its exit codes.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/USAGE.ru.md',
		slug: 'usage',
		locale: 'ru',
		dest: 'ru/usage.md',
		description: 'Все подкоманды, их опции и коды возврата.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/SHELL_INTEGRATION.md',
		slug: 'shell-integration',
		locale: 'en',
		dest: 'shell-integration.md',
		description: 'Wiring the shell function into zsh and bash so the session environment changes.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/SHELL_INTEGRATION.ru.md',
		slug: 'shell-integration',
		locale: 'ru',
		dest: 'ru/shell-integration.md',
		description: 'Подключение shell-функции в zsh и bash для смены окружения сессии.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'CONTRIBUTING.md',
		slug: 'contributing',
		locale: 'en',
		dest: 'contributing.md',
		description: 'Development setup, verification commands and the branching model.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'CHANGELOG.md',
		slug: 'changelog',
		locale: 'en',
		dest: 'changelog.md',
		description: 'Release history, following Keep a Changelog.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'CHANGELOG.ru.md',
		slug: 'changelog',
		locale: 'ru',
		dest: 'ru/changelog.md',
		description: 'История релизов в формате Keep a Changelog.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
];
```

- [ ] **Step 10: Add the test script**

In `website/package.json`, add to `"scripts"`:

```json
"test": "node --test \"scripts/**/*.test.mjs\""
```

The glob is quoted so Node expands it, not the shell. npm runs scripts under
`sh`, where `**` is not recursive — an unquoted glob silently matches only one
directory level, so a test file added directly under `scripts/` would never
run. Silently skipping tests is the same class of defect this plan exists to
remove.

Passing the directory (`node --test scripts/`) does not work here: on Node
26.7.0 it fails with a CJS `MODULE_NOT_FOUND` instead of recursing.

- [ ] **Step 11: Run the whole suite**

Run: `npm test`
Expected: PASS, 11 tests across two files.

- [ ] **Step 12: Commit**

```bash
git add website/scripts website/package.json
git commit -m "feat(website): add docs manifest with shape-asserting cleanup"
```

---

### Task 3: Link and asset resolution

Turns every internal link into either a base-prefixed site path or an absolute repository URL, and throws on anything else.

**Files:**
- Create: `website/scripts/lib/links.mjs`
- Create: `website/scripts/lib/links.test.mjs`

**Interfaces:**
- Consumes: `BASE`, `REPO_BLOB_URL` from Task 1; `PAGES` from Task 2.
- Produces: `links.mjs` exports `buildPageMap(pages: PageEntry[]): Map<string, string>`, `resolveLink(href: string, sourcePath: string, pageMap: Map<string,string>): string`, and `rewriteLinks(markdown: string, entry: PageEntry, pageMap: Map<string,string>): string`.

The link inventory this must cover, gathered from the sources:

| Source form | Resolves to | Becomes |
| :--- | :--- | :--- |
| `docs/INSTALLATION.md`, `INSTALLATION.md` | `docs/INSTALLATION.md` | `/Terminal-Session-Proxy-Manager/installation/` |
| `docs/INSTALLATION.ru.md`, `INSTALLATION.ru.md` | `docs/INSTALLATION.ru.md` | `/Terminal-Session-Proxy-Manager/ru/installation/` |
| `README.md` | `README.md` | `/Terminal-Session-Proxy-Manager/overview/` |
| `CHANGELOG.md`, `CONTRIBUTING.md` | same | `/…/changelog/`, `/…/contributing/` |
| `LICENSE`, `SECURITY.md`, `AGENTS.md`, `.ai/GIT_WORKFLOW.md` | same | `https://github.com/…/blob/main/<path>` |
| `configs/config.default.json`, `../configs/config.default.json` | `configs/config.default.json` | `https://github.com/…/blob/main/configs/config.default.json` |
| `../shell/terminal-session-proxy-manager.{zsh,bash}` | `shell/…` | `https://github.com/…/blob/main/shell/…` |
| `#-path-setup`, `#-настройка-path` | — | unchanged |
| `assets/banner_new.jpg`, `assets/proxy_dashboard_final.png` | — | depth-correct path into `src/assets/` |

- [ ] **Step 1: Write the failing tests**

```js
// website/scripts/lib/links.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';

import { PAGES } from '../docs-manifest.mjs';
import { buildPageMap, resolveLink, rewriteLinks } from './links.mjs';

const pageMap = buildPageMap(PAGES);

test('a sibling link inside docs/ resolves to an absolute site path', () => {
	assert.equal(
		resolveLink('INSTALLATION.md', 'docs/USAGE.md', pageMap),
		'/Terminal-Session-Proxy-Manager/installation/'
	);
});

test('a docs/ link from the repository root resolves to the same page', () => {
	assert.equal(
		resolveLink('docs/INSTALLATION.md', 'README.md', pageMap),
		'/Terminal-Session-Proxy-Manager/installation/'
	);
});

test('a Russian source maps to the Russian page', () => {
	assert.equal(
		resolveLink('SHELL_INTEGRATION.ru.md', 'docs/USAGE.ru.md', pageMap),
		'/Terminal-Session-Proxy-Manager/ru/shell-integration/'
	);
});

test('an anchor is preserved through resolution', () => {
	assert.equal(
		resolveLink('INSTALLATION.md#-path-setup', 'docs/USAGE.md', pageMap),
		'/Terminal-Session-Proxy-Manager/installation/#-path-setup'
	);
});

test('a bare in-page anchor is left alone', () => {
	assert.equal(resolveLink('#-path-setup', 'docs/USAGE.md', pageMap), '#-path-setup');
});

test('an external URL is left alone', () => {
	assert.equal(
		resolveLink('https://rustup.rs', 'CONTRIBUTING.md', pageMap),
		'https://rustup.rs'
	);
});

test('a repository file with no page becomes an absolute GitHub URL', () => {
	assert.equal(
		resolveLink('LICENSE', 'README.md', pageMap),
		'https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/blob/main/LICENSE'
	);
});

test('a parent-relative repository path is normalised before lookup', () => {
	assert.equal(
		resolveLink('../configs/config.default.json', 'docs/CONFIGURATION.md', pageMap),
		'https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/blob/main/configs/config.default.json'
	);
});

test('an unknown internal target throws rather than producing a dead link', () => {
	assert.throws(() => resolveLink('NOPE.md', 'README.md', pageMap), /NOPE\.md/);
});

test('an image path is rewritten to the website asset directory, depth-aware', () => {
	const en = rewriteLinks(
		'![B](assets/proxy_dashboard_final.png)',
		{ source: 'README.md', dest: 'overview.md' },
		pageMap
	);
	assert.equal(en, '![B](../../assets/proxy_dashboard_final.png)');

	const ru = rewriteLinks(
		'![B](assets/proxy_dashboard_final.png)',
		{ source: 'README.ru.md', dest: 'ru/overview.md' },
		pageMap
	);
	assert.equal(ru, '![B](../../../assets/proxy_dashboard_final.png)');
});

test('rewriteLinks leaves link text untouched', () => {
	const out = rewriteLinks(
		'See [the guide](docs/USAGE.md) for details.',
		{ source: 'README.md', dest: 'overview.md' },
		pageMap
	);
	assert.equal(out, 'See [the guide](/Terminal-Session-Proxy-Manager/usage/) for details.');
});
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `node --test scripts/lib/links.test.mjs`
Expected: FAIL — `Cannot find module './links.mjs'`

- [ ] **Step 3: Implement link resolution**

```js
// website/scripts/lib/links.mjs
import path from 'node:path';

import { BASE, REPO_BLOB_URL } from '../../site.config.mjs';

/**
 * Repository files that are legitimately linked from the docs but have no page
 * on the site. They resolve to GitHub rather than throwing.
 */
const REPO_FILE_PREFIXES = ['LICENSE', 'SECURITY.md', 'AGENTS.md', '.ai/', 'configs/', 'shell/'];

/** Map every synced source path to its absolute, base-prefixed site path. */
export function buildPageMap(pages) {
	const map = new Map();
	for (const page of pages) {
		const prefix = page.locale === 'ru' ? `${BASE}/ru` : BASE;
		map.set(page.source, `${prefix}/${page.slug}/`);
	}
	return map;
}

function isExternal(href) {
	return /^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith('//');
}

/**
 * Resolve one Markdown link target.
 *
 * Throws on anything unrecognised. The generator this replaces fell through to
 * the original text, which is how `href="CHANGELOG.md"` reached production.
 */
export function resolveLink(href, sourcePath, pageMap) {
	if (href.startsWith('#') || isExternal(href)) return href;

	const hashIndex = href.indexOf('#');
	const target = hashIndex === -1 ? href : href.slice(0, hashIndex);
	const hash = hashIndex === -1 ? '' : href.slice(hashIndex);

	if (target === '') return href;

	// Resolve relative to the source document, then normalise away any `../`.
	const resolved = path.posix
		.normalize(path.posix.join(path.posix.dirname(sourcePath), target))
		.replace(/^\.\//, '');

	const page = pageMap.get(resolved);
	if (page) return page + hash;

	if (REPO_FILE_PREFIXES.some((prefix) => resolved === prefix || resolved.startsWith(prefix))) {
		return REPO_BLOB_URL + resolved + hash;
	}

	throw new Error(
		`${sourcePath}: link target "${href}" (resolved to "${resolved}") is not a known page ` +
			`or repository file. Add it to the manifest or to REPO_FILE_PREFIXES.`
	);
}

/**
 * Rewrite every Markdown link and image in a document.
 *
 * Images under `assets/` point at copies in `src/assets/` so Astro can process
 * them; the number of `../` segments depends on how deep the generated page sits.
 */
export function rewriteLinks(markdown, entry, pageMap) {
	const depth = entry.dest.split('/').length - 1;
	const assetPrefix = '../'.repeat(depth + 2) + 'assets/';

	return markdown.replace(/(!?)\[([^\]]*)\]\(([^)\s]+)(\s+"[^"]*")?\)/g, (match, bang, text, href, title) => {
		const suffix = title ?? '';
		if (href.startsWith('assets/')) {
			return `${bang}[${text}](${assetPrefix}${href.slice('assets/'.length)}${suffix})`;
		}
		if (bang === '!') return match;
		return `[${text}](${resolveLink(href, entry.source, pageMap)}${suffix})`;
	});
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `node --test scripts/lib/links.test.mjs`
Expected: PASS, 11 tests.

- [ ] **Step 5: Commit**

```bash
git add website/scripts/lib/links.mjs website/scripts/lib/links.test.mjs
git commit -m "feat(website): resolve doc links to base-prefixed paths, fail on unknown targets"
```

---

### Task 4: Generator orchestration and Git-ignored output

Wires the modules together, deletes the old generator, and stops generated pages from being tracked.

**Files:**
- Create: `website/scripts/sync-docs.mjs`
- Delete: `website/scripts/sync-docs.js`
- Delete: `website/src/content/docs/guides/example.md`
- Delete: `website/src/content/docs/reference/example.md`
- Delete: `website/src/assets/houston.webp`
- Modify: `website/.gitignore`
- Modify: `website/package.json`

**Interfaces:**
- Consumes: `PAGES`, `applyCleanup`, `extractTitle`, `buildFrontmatter`, `buildPageMap`, `rewriteLinks`.
- Produces: `npm run sync` writing all thirteen pages into `src/content/docs/`.

- [ ] **Step 1: Write the orchestrator**

```js
// website/scripts/sync-docs.mjs
// Generates the site's content from the repository's canonical Markdown.
// Output is Git-ignored: the generated pages exist only inside a build, so
// they cannot drift from their sources.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { PAGES } from './docs-manifest.mjs';
import { applyCleanup, extractTitle } from './lib/cleanup.mjs';
import { buildFrontmatter } from './lib/frontmatter.mjs';
import { buildPageMap, rewriteLinks } from './lib/links.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '../../');
const CONTENT = path.join(ROOT, 'website/src/content/docs');
const ASSETS = path.join(ROOT, 'website/src/assets');

const pageMap = buildPageMap(PAGES);

for (const entry of PAGES) {
	const sourcePath = path.join(ROOT, entry.source);
	if (!fs.existsSync(sourcePath)) {
		throw new Error(`${entry.source}: listed in the manifest but missing from the repository.`);
	}

	const raw = fs.readFileSync(sourcePath, 'utf8');
	const cleaned = applyCleanup(raw, entry);
	const { title, body } = extractTitle(cleaned);
	const rewritten = rewriteLinks(body, entry, pageMap);

	for (const [, asset] of rewritten.matchAll(/\.\.\/assets\/([^)\s]+)/g)) {
		if (!fs.existsSync(path.join(ASSETS, asset))) {
			throw new Error(`${entry.source}: references assets/${asset}, missing from website/src/assets/.`);
		}
	}

	const frontmatter = buildFrontmatter({ title, description: entry.description });
	const destPath = path.join(CONTENT, entry.dest);
	fs.mkdirSync(path.dirname(destPath), { recursive: true });
	fs.writeFileSync(destPath, frontmatter + rewritten.trimStart() + '\n', 'utf8');
	console.log(`  ${entry.source} → ${entry.dest}`);
}

console.log(`Synced ${PAGES.length} pages.`);
```

- [ ] **Step 2: Point the npm scripts at the new file**

In `website/package.json`, replace `predev`/`prebuild` and add `sync`:

```json
"sync": "node scripts/sync-docs.mjs",
"predev": "node scripts/sync-docs.mjs",
"prebuild": "node scripts/sync-docs.mjs",
```

- [ ] **Step 3: Delete the old generator and template leftovers**

```bash
rm website/scripts/sync-docs.js
rm -rf website/src/content/docs/guides website/src/content/docs/reference
rm website/src/assets/houston.webp
```

- [ ] **Step 4: Ignore generated content**

Append to `website/.gitignore`:

```gitignore
# Generated from the repository's canonical Markdown by scripts/sync-docs.mjs.
# Never edit these by hand and never commit them — edit the source documents
# listed in scripts/docs-manifest.mjs instead.
src/content/docs/*.md
src/content/docs/ru/*.md
```

Then untrack any generated page that is already staged. Scope this to the
generated `.md` files: `index.mdx` and `ru/index.mdx` are hand-authored and
must stay tracked, so never run this against the directory as a whole.

```bash
git rm --cached --ignore-unmatch \
  'website/src/content/docs/*.md' 'website/src/content/docs/ru/*.md'
```

- [ ] **Step 5: Run the generator and verify it succeeds**

Run:
```bash
npm run sync
```
Expected: thirteen `→` lines and `Synced 13 pages.` with no exception.

- [ ] **Step 6: Verify the previously broken links are gone**

Run:
```bash
npm run build
grep -oE 'href="(CHANGELOG\.md|CONTRIBUTING\.md|LICENSE|installation)"' dist/overview/index.html || echo "OK: no bare repo-file links"
```
Expected: `OK: no bare repo-file links`

- [ ] **Step 7: Commit**

```bash
git add -A website/scripts website/.gitignore website/package.json website/src
git commit -m "refactor(website): rebuild docs generator, ignore generated content"
```

---

### Task 5: Styling without the theme fight

**Files:**
- Modify: `website/src/styles/custom.css` (full rewrite)
- Modify: `website/package.json`

**Interfaces:**
- Consumes: nothing.
- Produces: nothing other tasks read.

- [ ] **Step 1: Install self-hosted fonts**

```bash
npm install @fontsource-variable/inter @fontsource-variable/jetbrains-mono
```

- [ ] **Step 2: Rewrite the stylesheet**

Replace the whole of `website/src/styles/custom.css`. The `!important` declarations in the previous version existed only to out-rank `starlight-theme-md3`, which is gone; none are needed now.

```css
/*
 * A thin layer over stock Starlight. Everything here uses Starlight's
 * documented custom properties, so there is nothing to out-rank and no
 * `!important` anywhere in this file. Keep it that way: an `!important` here
 * means a property is being fought rather than set.
 */
@import '@fontsource-variable/inter';
@import '@fontsource-variable/jetbrains-mono';

:root {
	--sl-font: 'Inter Variable', ui-sans-serif, system-ui, sans-serif;
	--sl-font-mono: 'JetBrains Mono Variable', ui-monospace, SFMono-Regular, monospace;

	/* Rust orange, matching the project banner. */
	--sl-hue-accent: 25;
	--sl-color-accent-low: hsl(var(--sl-hue-accent) 70% 20%);
	--sl-color-accent: hsl(var(--sl-hue-accent) 80% 50%);
	--sl-color-accent-high: hsl(var(--sl-hue-accent) 90% 80%);
}

:root[data-theme='light'] {
	--sl-color-accent-low: hsl(var(--sl-hue-accent) 90% 88%);
	--sl-color-accent: hsl(var(--sl-hue-accent) 85% 45%);
	--sl-color-accent-high: hsl(var(--sl-hue-accent) 80% 25%);
}

/* Translucent header, so content scrolling under it stays legible. */
header.header {
	background-color: color-mix(in srgb, var(--sl-color-bg-nav) 80%, transparent);
	backdrop-filter: blur(12px);
	-webkit-backdrop-filter: blur(12px);
}

.sl-markdown-content img {
	border-radius: 0.5rem;
}

/* Screenshots of the TUI are the one place where a border helps. */
.sl-markdown-content img[src*='proxy_dashboard'] {
	border: 1px solid var(--sl-color-hairline);
}

::-webkit-scrollbar {
	width: 10px;
	height: 10px;
}
::-webkit-scrollbar-track {
	background: transparent;
}
::-webkit-scrollbar-thumb {
	background: var(--sl-color-gray-5);
	border: 2px solid var(--sl-color-bg);
	border-radius: 10px;
}
::-webkit-scrollbar-thumb:hover {
	background: var(--sl-color-gray-4);
}

@media (prefers-reduced-motion: no-preference) {
	html {
		scroll-behavior: smooth;
	}
}
```

- [ ] **Step 3: Verify no `!important` survives and the build passes**

Run:
```bash
grep -c '!important' src/styles/custom.css || echo "OK: zero !important"
npm run build
```
Expected: `OK: zero !important`, then a successful build.

- [ ] **Step 4: Commit**

```bash
git add website/src/styles/custom.css website/package.json website/package-lock.json
git commit -m "style(website): replace md3 overrides with stock Starlight properties"
```

---

### Task 6: Stop the theme toggle from disabling page transitions

`ThemeSelect.astro` sets `animation: none` on `::view-transition-old(root)` and `::view-transition-new(root)` in a `:global` block. Those pseudo-elements are also what `astro-vtbot` animates between pages, so the page transitions are dead. Scoping the override to an attribute present only during a theme change fixes both.

**Files:**
- Modify: `website/src/components/ThemeSelect.astro`

**Interfaces:**
- Consumes: nothing.
- Produces: nothing.

- [ ] **Step 1: Scope the view-transition overrides**

In the `<style>` block, replace the three `:global(::view-transition-…)` rules with:

```css
	/*
	 * These pseudo-elements are shared with astro-vtbot's page transitions.
	 * Scoping the override to an attribute set only while the theme changes
	 * keeps the circular reveal without killing navigation animations.
	 */
	:global([data-theme-transition]::view-transition-old(root)),
	:global([data-theme-transition]::view-transition-new(root)) {
		animation: none;
		mix-blend-mode: normal;
	}
	:global([data-theme-transition]::view-transition-old(root)) {
		z-index: 1;
	}
	:global([data-theme-transition]::view-transition-new(root)) {
		z-index: 9999;
	}
```

- [ ] **Step 2: Set and clear the attribute around the transition**

In the click handler, replace the `document.startViewTransition(...)` block with:

```js
					document.documentElement.dataset.themeTransition = '';
					const transition = document.startViewTransition(() => {
						setTheme(newTheme);
					});

					transition.finished.finally(() => {
						delete document.documentElement.dataset.themeTransition;
					});

					transition.ready.then(() => {
						document.documentElement.animate(
							{
								clipPath: [
									`circle(0px at ${x}px ${y}px)`,
									`circle(${endRadius}px at ${x}px ${y}px)`,
								],
							},
							{
								duration: 500,
								easing: 'ease-in-out',
								pseudoElement: '::view-transition-new(root)',
							}
						);
					});
```

- [ ] **Step 3: Verify both animations manually**

Run: `npm run dev`

Open `http://localhost:4321/Terminal-Session-Proxy-Manager/` and check:
1. Clicking a sidebar link fades between pages (astro-vtbot works again).
2. Clicking the light/dark toggle reveals the new theme as an expanding circle from the cursor.
3. `document.documentElement.dataset.themeTransition` is absent once the reveal finishes.

- [ ] **Step 4: Commit**

```bash
git add website/src/components/ThemeSelect.astro
git commit -m "fix(website): stop theme toggle from disabling page transitions"
```

---

### Task 7: Plugins, social image and landing links

**Files:**
- Modify: `website/astro.config.mjs`
- Modify: `website/src/content/docs/index.mdx`
- Modify: `website/src/content/docs/ru/index.mdx`
- Modify: `website/package.json`

**Interfaces:**
- Consumes: `SITE`, `BASE` from Task 1.
- Produces: nothing.

- [ ] **Step 1: Install the plugins**

```bash
npm install starlight-links-validator starlight-image-zoom starlight-llms-txt
```

- [ ] **Step 2: Read the link validator's options before configuring it**

Run: `cat node_modules/starlight-links-validator/README.md`

Confirm how it treats a configured `base` and which option controls relative links. Configure accordingly in the next step; if its defaults already reject relative links, pass no options beyond `errorOnRelativeLinks`.

- [ ] **Step 3: Register plugins and the social image**

In `website/astro.config.mjs`, extend the imports:

```js
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightLlmsTxt from 'starlight-llms-txt';
```

Replace the `plugins` array:

```js
			plugins: [
				viewTransitions(),
				starlightImageZoom(),
				starlightLlmsTxt(),
				// A dead internal link fails the build. This is the gate that would
				// have caught the `href="CHANGELOG.md"` links shipped previously.
				starlightLinksValidator({ errorOnRelativeLinks: true }),
			],
```

Add a `head` entry beside `customCss`. Starlight already emits canonical,
`og:title`, `og:type`, `og:url`, `og:locale`, `og:description`, `og:site_name`
and `twitter:card`; `og:image` is the only tag it never generates.

```js
			head: [
				{
					tag: 'meta',
					attrs: { property: 'og:image', content: `${SITE}${BASE}/og.jpg` },
				},
				{
					tag: 'meta',
					attrs: { name: 'twitter:image', content: `${SITE}${BASE}/og.jpg` },
				},
			],
```

- [ ] **Step 4: Provide the social image**

```bash
cp ../assets/banner_new.jpg public/og.jpg
```

`public/` is served from the site root under `base`, so `og.jpg` resolves to
`/Terminal-Session-Proxy-Manager/og.jpg`, matching the tags above.

- [ ] **Step 5: Make the landing hero links survive `base`**

Hero action links are passed straight to an `<a>` without base prefixing —
confirmed both in `node_modules/@astrojs/starlight/components/Hero.astro` and
in Starlight's own reference. They must therefore carry the base themselves.

In `website/src/content/docs/index.mdx`, change the three `link:` values:

The link validator configured in Step 3 rejects relative links, so these are
written as absolute paths that already carry the base. The literal duplicates
`BASE`, which MDX frontmatter cannot import — but the duplication is
self-policing: if `BASE` ever changes, these links stop resolving and the
validator fails the build.

```yaml
    - text: Get Started
      link: /Terminal-Session-Proxy-Manager/overview/
      icon: right-arrow
    - text: Read the Docs
      link: /Terminal-Session-Proxy-Manager/installation/
      variant: minimal
      icon: document
```

In `website/src/content/docs/ru/index.mdx`:

```yaml
    - text: Начать работу
      link: /Terminal-Session-Proxy-Manager/ru/overview/
      icon: right-arrow
    - text: Документация
      link: /Terminal-Session-Proxy-Manager/ru/installation/
      variant: minimal
      icon: document
```

Leave the GitHub action untouched — it is already absolute.

- [ ] **Step 6: Build and confirm the validator is active**

Run:
```bash
npm run build
```
Expected: the build log includes a `starlight-links-validator` line and completes without errors.

- [ ] **Step 7: Confirm the hero links and social tags are correct**

Run:
```bash
grep -o 'href="/Terminal-Session-Proxy-Manager/overview/"' dist/index.html
grep -o 'property="og:image" content="[^"]*"' dist/index.html
```
Expected: both match, with `og:image` pointing at `https://lebedevkondakovsergeyvach.github.io/Terminal-Session-Proxy-Manager/og.jpg`.

- [ ] **Step 8: Commit**

```bash
git add website/astro.config.mjs website/src/content/docs/index.mdx website/src/content/docs/ru/index.mdx website/public/og.jpg website/package.json website/package-lock.json
git commit -m "feat(website): add link validation, image zoom, llms.txt and social image"
```

---

### Task 8: Documentation and agent configuration

The current `website/AGENTS.md` instructs agents to run `astro dev --background` and `astro dev stop|status|logs`. None of those exist — `npx astro --help` on v7.2.4 lists only `add`, `build`, `check`, `create-key`, `dev`, `docs`, `info`, `preview`, `sync`, `preferences`, `telemetry`. `website/README.md` documents `variant: 'expressive'` where the config said `fidelity`, and a `Head.astro` that was deleted.

**Files:**
- Create: `.mcp.json` (repository root — the only file outside `website/`)
- Modify: `website/AGENTS.md` (full rewrite)
- Modify: `website/README.md` (full rewrite)

**Interfaces:**
- Consumes: everything above.
- Produces: nothing.

- [ ] **Step 1: Add the MCP server configuration**

Claude Code reads `.mcp.json` at the repository root. `.agents/mcp_config.json`
is read by nothing, which is why the existing instruction to "always query
astro-docs" was unfollowable. The GitHub and Brave entries are dropped: both
carry placeholder credentials, and neither is needed for this work.

```json
{
  "mcpServers": {
    "astro-docs": {
      "type": "http",
      "url": "https://mcp.docs.astro.build/mcp"
    },
    "context7": {
      "type": "http",
      "url": "https://mcp.context7.com/mcp"
    }
  }
}
```

- [ ] **Step 2: Rewrite `website/AGENTS.md`**

```markdown
# AGENTS.md — website

Instructions for AI agents working in `website/`. The rest of the repository is
covered by the root [`AGENTS.md`](../AGENTS.md).

## What this is

An Astro Starlight documentation site, deployed as a GitHub Pages **project
page** at `/Terminal-Session-Proxy-Manager/`. Bilingual: English at the root,
Russian under `/ru/`.

## Commands

Run everything from `website/`.

```bash
npm install      # install dependencies
npm run sync     # regenerate content from the repository's Markdown
npm run dev      # dev server (runs sync first)
npm run build    # production build (runs sync first, validates every link)
npm run preview  # serve the built site
npm test         # unit tests for the generator
```

There is no background mode. `astro dev` runs in the foreground; use your
harness's background execution if you need the shell back.

## Rules

### 1. Never edit the generated pages

`src/content/docs/*.md` and `src/content/docs/ru/*.md` are generated by
`scripts/sync-docs.mjs` and are Git-ignored. Edits are erased by the next
build.

The sources are the repository's canonical Markdown — `README.md`,
`README.ru.md`, `docs/*.md`, `CONTRIBUTING.md`, `CHANGELOG*.md` — listed in
`scripts/docs-manifest.mjs`.

Hand-authored pages are the exception: `index.mdx` and `ru/index.mdx`.

### 2. Adding a page means editing the manifest

Add an entry to `PAGES` in `scripts/docs-manifest.mjs` with its slug, locale,
destination, description and cleanup flags, then add it to the `sidebar` in
`astro.config.mjs` for both locales.

### 3. The generator throws on purpose

An unrecognised link target, a missing asset, or a source document whose shape
changed all stop the build. This is deliberate: the generator this replaced
fell through silently and shipped dead links. Fix the manifest — do not add a
fallback.

### 4. Links must survive `base`

The site is served from a sub-path. Starlight prefixes `base` automatically for
sidebar entries and the site title, and for nothing else — hero action links in
particular are passed straight to an `<a>`.

- In generated content, links go through `scripts/lib/links.mjs`, which emits
  base-prefixed absolute paths. Never hard-code one.
- In hand-authored `.mdx`, use relative targets (`overview/`), which resolve
  correctly under any base.
- `site` and `base` have one definition: `site.config.mjs`.

### 5. Both locales, always

Every content change lands in English and Russian. `contributing` is the sole
exception — there is no `CONTRIBUTING.ru.md`, so Starlight's fallback serves
the English page and the sidebar carries a badge saying so.

### 6. No `!important` in `custom.css`

The stylesheet sets Starlight's documented custom properties. An `!important`
means something is being fought rather than configured, which is what the
removed Material Design 3 theme required.

### 7. Client scripts and page transitions

`astro-vtbot` gives the site SPA-style navigation, so `<script>` tags do not
re-run on soft navigations. Listen for `astro:page-load`, or define a custom
element as `ThemeSelect.astro` does.

`ThemeSelect.astro` animates `::view-transition-*(root)`, the same
pseudo-elements astro-vtbot uses. Its overrides are scoped behind
`[data-theme-transition]` so the two coexist. Do not widen that scope.

## MCP servers

`.mcp.json` at the repository root configures `astro-docs` and `context7`. Use
them for Astro and Starlight APIs rather than recalling them — this ecosystem
moves fast, and the installed versions are Astro 7.2.4 with Starlight 0.41.7.
```

- [ ] **Step 3: Rewrite `website/README.md`**

```markdown
# Documentation website

The [Astro Starlight](https://starlight.astro.build/) documentation site for
Terminal Session Proxy Manager. Deployed as a GitHub Pages project page at
`/Terminal-Session-Proxy-Manager/`, in English and Russian.

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
- `scripts/lib/links.mjs` rewrites every link to a base-prefixed site path or
  an absolute GitHub URL, and throws on anything unrecognised.
- `scripts/lib/frontmatter.mjs` serialises frontmatter as valid YAML.

Generated pages are Git-ignored, so the site cannot drift from its sources.

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
```

- [ ] **Step 4: Verify the documented commands actually run**

Run:
```bash
npm run sync && npm test && npm run build
```
Expected: all three succeed.

- [ ] **Step 5: Commit**

```bash
git add .mcp.json website/AGENTS.md website/README.md
git commit -m "docs(website): correct agent instructions and add MCP configuration"
```

---

### Task 9: Full verification

**Files:** none.

- [ ] **Step 1: Build from a clean state**

```bash
rm -rf dist .astro src/content/docs/*.md src/content/docs/ru/*.md
npm run build
```
Expected: sync reports thirteen pages, the link validator reports no errors, the build succeeds.

- [ ] **Step 2: Confirm every page exists in both locales**

```bash
for slug in overview installation configuration usage shell-integration changelog contributing; do
  test -f "dist/$slug/index.html" && echo "en/$slug ok" || echo "en/$slug MISSING"
done
for slug in overview installation configuration usage shell-integration changelog; do
  test -f "dist/ru/$slug/index.html" && echo "ru/$slug ok" || echo "ru/$slug MISSING"
done
```
Expected: thirteen `ok` lines, no `MISSING`.

- [ ] **Step 3: Confirm the template leftovers are gone**

```bash
test -d dist/guides -o -d dist/reference && echo "LEFTOVERS PRESENT" || echo "OK: clean"
```
Expected: `OK: clean`

- [ ] **Step 4: Confirm no link escaped rewriting**

BSD grep on macOS has no `-P`, so match the failure shapes directly rather
than with a negative lookahead:

```bash
grep -rhoE 'href="[^"]*\.md"' dist --include=index.html | sort -u
grep -rhoE 'href="(LICENSE|SECURITY|AGENTS|configs/|shell/)[^"]*"' dist --include=index.html | sort -u
```
Expected: both produce no output. Any hit is a link the generator failed to
rewrite.

- [ ] **Step 5: Preview under the real base path**

```bash
npm run preview
```
Open `http://localhost:4321/Terminal-Session-Proxy-Manager/`. Check the sidebar
in both locales, the language switcher, search, the theme toggle reveal, and a
page-to-page navigation fade.

- [ ] **Step 6: Report**

Summarise what was verified with the actual command output. Do not claim a
green build without showing it.
