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
 * Split markdown into segments that must never be touched by link rewriting
 * (fenced code blocks) and segments that may be.
 *
 * A fence opens on a line with up to 3 leading spaces followed by 3+ backticks
 * or 3+ tildes (CommonMark's rule), and closes on a line carrying a fence of
 * the same character at least as long. An unterminated fence runs to the end
 * of the document — code, not a truncation to fix up here.
 */
function splitFencedBlocks(markdown) {
	const segments = [];
	const fenceStartRe = /^ {0,3}(`{3,}|~{3,})[^\n]*$/;
	const len = markdown.length;
	let lineStart = 0;

	const pushText = (value) => {
		if (value === '') return;
		const last = segments[segments.length - 1];
		if (last && last.type === 'text') last.value += value;
		else segments.push({ type: 'text', value });
	};

	while (lineStart < len) {
		let lineEnd = markdown.indexOf('\n', lineStart);
		const hasNewline = lineEnd !== -1;
		if (!hasNewline) lineEnd = len;
		const line = markdown.slice(lineStart, lineEnd);
		const fenceMatch = line.match(fenceStartRe);

		if (fenceMatch) {
			const fenceChar = fenceMatch[1][0];
			const fenceLen = fenceMatch[1].length;
			const blockStart = lineStart;
			const closeRe = new RegExp(`^ {0,3}${fenceChar}{${fenceLen},}\\s*$`);
			let cursor = hasNewline ? lineEnd + 1 : len;
			let blockEnd = len;

			while (cursor < len) {
				let cEnd = markdown.indexOf('\n', cursor);
				const cHasNewline = cEnd !== -1;
				if (!cHasNewline) cEnd = len;
				const cLine = markdown.slice(cursor, cEnd);
				if (closeRe.test(cLine)) {
					blockEnd = cHasNewline ? cEnd + 1 : len;
					break;
				}
				cursor = cHasNewline ? cEnd + 1 : len;
			}

			segments.push({ type: 'code', value: markdown.slice(blockStart, blockEnd) });
			lineStart = blockEnd;
			continue;
		}

		const textEnd = hasNewline ? lineEnd + 1 : len;
		pushText(markdown.slice(lineStart, textEnd));
		lineStart = textEnd;
	}

	return segments;
}

/**
 * If `text[i]` opens a run of backticks with a matching close of the same
 * length, return the index just past that close. Otherwise return -1 — an
 * unmatched run of backticks is literal text, not a code span.
 */
function matchInlineCode(text, i) {
	if (text[i] !== '`') return -1;

	let j = i;
	while (text[j] === '`') j += 1;
	const tickLen = j - i;

	const closeRe = new RegExp('`{' + tickLen + '}(?!`)', 'g');
	closeRe.lastIndex = j;
	const closeMatch = closeRe.exec(text);
	return closeMatch ? closeMatch.index + closeMatch[0].length : -1;
}

/**
 * Index of the character that closes `openChar` opened at `openIndex`, or -1.
 *
 * Inline code spans are skipped atomically: a bracket-like character inside
 * one (e.g. the `]` in `` `a[b]` ``) must not affect the depth count, and a
 * link's display text may itself be entirely or partly backtick-wrapped
 * (`` [`path`](href) ``) without breaking the match.
 */
function findMatchingBracket(text, openIndex, openChar, closeChar) {
	let depth = 0;
	let i = openIndex;

	while (i < text.length) {
		if (text[i] === '\\') {
			i += 2;
			continue;
		}

		const codeEnd = matchInlineCode(text, i);
		if (codeEnd !== -1) {
			i = codeEnd;
			continue;
		}

		if (text[i] === openChar) {
			depth += 1;
		} else if (text[i] === closeChar) {
			depth -= 1;
			if (depth === 0) return i;
		}

		i += 1;
	}

	return -1;
}

function renderLinkOrImage(isImage, innerText, hrefRaw, entry, pageMap, assetPrefix) {
	const parts = hrefRaw.match(/^(\S+)(\s+"[^"]*")?$/);
	const href = parts ? parts[1] : hrefRaw;
	const title = parts && parts[2] ? parts[2] : '';

	if (isImage) {
		if (href.startsWith('assets/')) {
			return `![${innerText}](${assetPrefix}${href.slice('assets/'.length)}${title})`;
		}
		return `![${innerText}](${href}${title})`;
	}

	// Link text can itself be an image (`[![alt](img)](target)`); rewrite it
	// too rather than passing it through untouched.
	const rewrittenInner = rewriteTextRun(innerText, entry, pageMap, assetPrefix);
	return `[${rewrittenInner}](${resolveLink(href, entry.source, pageMap)}${title})`;
}

/**
 * Rewrite links and images in a run of text known to contain no fenced code
 * blocks (inline code spans are handled here, atomically). Walks the text by
 * hand rather than with a single regex because a link's text can itself
 * contain a bracket-paren image (`[![alt](img)](target)`), which a
 * non-recursive regex cannot balance.
 *
 * Inline code is resolved before link detection at each position: a run of
 * backticks that opens before any `[`/`![` (e.g. `` `[nope](NOPE.md)` ``) is
 * consumed as one opaque, unrewritten span, exactly as if it were never link
 * syntax. A `[`/`![` reached without first crossing an unmatched backtick is
 * eligible for link detection, and `findMatchingBracket` skips any inline
 * code nested inside its display text (e.g. `` [`path`](href) ``) the same
 * way, so a code-formatted link label doesn't hide the link's href from
 * rewriting.
 */
function rewriteTextRun(text, entry, pageMap, assetPrefix) {
	let out = '';
	let i = 0;

	while (i < text.length) {
		const codeEnd = matchInlineCode(text, i);
		if (codeEnd !== -1) {
			out += text.slice(i, codeEnd);
			i = codeEnd;
			continue;
		}

		const isImage = text[i] === '!' && text[i + 1] === '[';
		const bracketStart = isImage ? i + 1 : text[i] === '[' ? i : -1;

		if (bracketStart !== -1) {
			const closeBracket = findMatchingBracket(text, bracketStart, '[', ']');
			if (closeBracket !== -1 && text[closeBracket + 1] === '(') {
				const closeParen = findMatchingBracket(text, closeBracket + 1, '(', ')');
				if (closeParen !== -1) {
					const innerText = text.slice(bracketStart + 1, closeBracket);
					const hrefRaw = text.slice(closeBracket + 2, closeParen);
					out += renderLinkOrImage(isImage, innerText, hrefRaw, entry, pageMap, assetPrefix);
					i = closeParen + 1;
					continue;
				}
			}
		}

		out += text[i];
		i += 1;
	}

	return out;
}

/**
 * Rewrite every Markdown link and image in a document.
 *
 * Fenced code blocks and inline code spans are left byte-for-byte untouched
 * and never reach link resolution — a bracket-paren shape inside an example
 * command is not a link, and must not throw just because it isn't one.
 * Images under `assets/` point at copies in `src/assets/` so Astro can
 * process them; the number of `../` segments depends on how deep the
 * generated page sits. A link whose text is itself an image gets both parts
 * rewritten.
 */
export function rewriteLinks(markdown, entry, pageMap) {
	const depth = entry.dest.split('/').length - 1;
	const assetPrefix = '../'.repeat(depth + 2) + 'assets/';

	return splitFencedBlocks(markdown)
		.map((segment) =>
			segment.type === 'code' ? segment.value : rewriteTextRun(segment.value, entry, pageMap, assetPrefix)
		)
		.join('');
}
