// website/scripts/sync-docs.mjs
// Generates the site's content from the repository's canonical Markdown.
// Output is Git-ignored: the generated pages exist only inside a build, so
// they cannot drift from their sources.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { PAGES } from './docs-manifest.mjs';
import { applyCleanup, extractTitle } from './lib/cleanup.mjs';
import { buildImports, expandDirectives, usedComponents } from './lib/components.mjs';
import { buildFrontmatter } from './lib/frontmatter.mjs';
import { buildPageMap, rewriteLinks, splitFencedBlocks } from './lib/links.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '../../');
const CONTENT = path.join(ROOT, 'website/src/content/docs');
const ASSETS = path.join(ROOT, 'website/src/assets');

const pageMap = buildPageMap(PAGES);

/** Text outside every fenced code block, for checks that must ignore examples. */
function proseOnly(markdown) {
	return splitFencedBlocks(markdown)
		.filter((segment) => segment.type === 'text')
		.map((segment) => segment.value)
		.join('');
}

for (const entry of PAGES) {
	const sourcePath = path.join(ROOT, entry.source);
	if (!fs.existsSync(sourcePath)) {
		throw new Error(`${entry.source}: listed in the manifest but missing from the repository.`);
	}

	const raw = fs.readFileSync(sourcePath, 'utf8');
	const cleaned = applyCleanup(raw, entry);
	const { title, body } = extractTitle(cleaned);
	const expanded = expandDirectives(body, entry.source);
	const rewritten = rewriteLinks(expanded, entry, pageMap);

	// The manifest declares the imports; the body decides which are real. A
	// mismatch either ships an unused import or an undefined component, so both
	// directions are errors rather than something to reconcile silently.
	const declared = new Set(entry.components ?? []);
	const used = usedComponents(proseOnly(rewritten));
	for (const name of used) {
		if (!declared.has(name)) {
			throw new Error(
				`${entry.source}: uses <${name}> but the manifest does not list it in \`components\`.`
			);
		}
	}
	for (const name of declared) {
		if (!used.has(name)) {
			throw new Error(`${entry.source}: manifest lists component "${name}", which the page never uses.`);
		}
	}

	// A `#` or `?` suffix is part of the URL, not the filename; the check runs
	// over prose only so an `../assets/…` path inside an example command is not
	// mistaken for a reference the site has to resolve.
	for (const [, reference] of proseOnly(rewritten).matchAll(/\.\.\/assets\/([^)\s]+)/g)) {
		const asset = reference.split(/[#?]/)[0];
		if (!fs.existsSync(path.join(ASSETS, asset))) {
			throw new Error(`${entry.source}: references assets/${asset}, missing from website/src/assets/.`);
		}
	}

	const frontmatter = buildFrontmatter({ title, description: entry.description });
	const imports = buildImports(entry.components);
	const destPath = path.join(CONTENT, entry.dest);
	fs.mkdirSync(path.dirname(destPath), { recursive: true });
	fs.writeFileSync(destPath, frontmatter + imports + rewritten.trimStart() + '\n', 'utf8');
	console.log(`  ${entry.source} → ${entry.dest}`);
}

console.log(`Synced ${PAGES.length} pages.`);
