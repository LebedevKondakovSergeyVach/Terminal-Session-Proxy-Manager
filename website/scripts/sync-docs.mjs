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
