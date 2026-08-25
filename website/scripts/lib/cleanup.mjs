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
