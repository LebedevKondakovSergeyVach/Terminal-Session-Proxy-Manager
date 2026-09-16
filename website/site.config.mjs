// website/site.config.mjs
import { readFileSync } from 'node:fs';

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

/**
 * The released version, read from the crate.
 *
 * `Cargo.toml` is the one place the version is allowed to live — the release
 * workflow tags whatever it says. The home pages used to hard-code it in three
 * places each, so both advertised v2.2.2 until someone remembered six literals.
 *
 * Only ever called from `astro.config.mjs`, which runs in Node. This module is
 * also imported by `.mdx` pages, and those get bundled: a relative `readFileSync`
 * there resolves against the bundle's location, not the source tree, and fails.
 * The config publishes the result as `PUBLIC_TSPM_VERSION` for the pages to read.
 */
export function readVersion() {
	const manifest = readFileSync(new URL('../Cargo.toml', import.meta.url), 'utf8');
	const match = manifest.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
	if (!match) throw new Error('Cargo.toml: no [package] version to read.');
	return match[1];
}
