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
