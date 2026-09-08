import { splitFencedBlocks } from './links.mjs';

/**
 * Hosts that only ever serve status badges. A line consisting solely of images
 * from these hosts is chrome for GitHub readers and noise on the site.
 */
const BADGE_PATTERNS = [/img\.shields\.io/, /komarev\.com/, /\/badge\.svg/];

/** How far into a document the H1 is allowed to appear. */
const H1_SEARCH_LINES = 15;

/**
 * Mark every line that sits inside a fenced code block.
 *
 * A `# ` shell comment in an example is not a heading. Without this the H1
 * search below would happily pick one up, and `stripPreamble` would then slice
 * the document from the middle of a fence — deleting the opening backticks and
 * swallowing the rest of the page into a code block, with nothing to fail on.
 */
function fencedLines(lines) {
	const fenced = new Array(lines.length).fill(false);
	let fence = null;

	for (let i = 0; i < lines.length; i += 1) {
		const match = lines[i].match(/^ {0,3}(`{3,}|~{3,})/);
		if (fence === null) {
			if (match) fence = match[1];
		} else {
			fenced[i] = true;
			if (match && match[1][0] === fence[0] && match[1].length >= fence.length && lines[i].trim() === match[1]) {
				fence = null;
			}
		}
	}

	return fenced;
}

/** Index of the first ATX H1 that is not inside a fence, or -1. */
function findHeading(lines, limit = Infinity) {
	const fenced = fencedLines(lines);
	for (let i = 0; i < lines.length && i < limit; i += 1) {
		if (!fenced[i] && /^#\s+\S/.test(lines[i])) return i;
	}
	return -1;
}

/**
 * Split the first H1 off the document.
 *
 * Starlight renders the title from frontmatter, so leaving the H1 in the body
 * would render it twice.
 */
export function extractTitle(markdown) {
	const lines = markdown.split('\n');
	const index = findHeading(lines, H1_SEARCH_LINES);
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
	return /^!\[[^\]]*\]\(assets\/banner\.jpg\)\s*$/.test(line.trim());
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
		const h1 = findHeading(lines);
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

	// Collapsing runs of blank lines is a prose tidy-up. Applied to the whole
	// document it would also reflow code, where blank lines carry meaning — a
	// diff, or a snippet spaced to PEP 8. So fences are excluded.
	return splitFencedBlocks(lines.join('\n'))
		.map((segment) => (segment.type === 'code' ? segment.value : segment.value.replace(/\n{3,}/g, '\n\n')))
		.join('')
		.trimStart();
}
