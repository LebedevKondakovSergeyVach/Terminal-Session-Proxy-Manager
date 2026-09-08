// website/scripts/lib/components.mjs
//
// Expands `<!--site:…-->` directives into Starlight components.
//
// The canonical documents in `docs/`, `CONTRIBUTING.md` and the READMEs are
// read on GitHub as well as on this site, so they must stay plain CommonMark.
// An earlier version of this branch put the JSX directly in them: every
// document opened with a literal `import { Tabs } from …` line that GitHub
// renders as a paragraph of JavaScript, and the components themselves were
// stripped by GitHub's sanitiser — taking the text in their attributes with
// them. `<Card title="Session Control">` left no trace of "Session Control",
// and `<TabItem label="Homebrew">` cost INSTALLATION.md three of its headings.
//
// So the structure lives in the source as ordinary headings, and the site
// wrapping lives in HTML comments, which GitHub renders as nothing at all.
// A `tabs` or `cards` region turns each of its headings into a tab label or a
// card title and consumes the heading line, so the title is not printed twice
// on the site. Everything else is a pass-through wrapper.
//
// Nothing here guesses. An unknown directive, an unclosed region, a region
// whose shape does not fit its mode — all throw, for the reason the rest of
// the generator throws: the generator this replaced fell through silently and
// shipped damaged pages.

/** `<!--site:name key="value" flag-->` or `<!--site:/name-->`, alone on a line. */
const DIRECTIVE_RE = /^<!--\s*site:(\/?)([a-zA-Z][\w-]*)((?:\s+[^>]*?)?)\s*-->$/;

/** Inline form, anywhere in a line: `<!--site:badge text="Core"-->`. */
const INLINE_DIRECTIVE_RE = /<!--\s*site:([a-zA-Z][\w-]*)((?:\s+[^>]*?)?)\s*-->/g;

/** A fence opens on ≤3 spaces of indent then 3+ backticks or tildes. */
const FENCE_RE = /^ {0,3}(`{3,}|~{3,})/;

/** ATX heading: 1–6 hashes, a space, then text. */
const HEADING_RE = /^(#{1,6})\s+(.*\S)\s*$/;

/** Extra props for the section a split region is currently building. */
const ITEM_RE = /^<!--\s*site:item((?:\s+[^>]*?)?)\s*-->$/;

/**
 * Region directives, and the component each one becomes.
 *
 * `split` regions divide their body at headings; the heading text becomes the
 * named prop and the heading line itself is dropped. `wrap` regions pass their
 * body through unchanged inside a single element.
 */
const REGIONS = {
	tabs: { kind: 'split', outer: 'Tabs', inner: 'TabItem', prop: 'label' },
	cards: { kind: 'split', outer: 'CardGrid', inner: 'Card', prop: 'title' },
	steps: { kind: 'wrap', outer: 'Steps' },
	filetree: { kind: 'wrap', outer: 'FileTree' },
	aside: { kind: 'wrap', outer: 'Aside' },
};

/** Directives with no body, replaced in place. */
const INLINE = {
	badge: 'Badge',
};

/**
 * Serialise a directive's attributes back into JSX.
 *
 * Accepts `key="value"` and bare flags (`stagger`), the two forms the Starlight
 * components in use here need. Values are kept verbatim inside double quotes,
 * so a value containing a `"` is rejected rather than silently truncated.
 */
export function parseAttributes(raw, context) {
	const text = raw.trim();
	if (text === '') return '';

	const attrs = [];
	const re = /([a-zA-Z][\w-]*)(?:=(?:"([^"]*)"|'([^']*)'))?/g;
	let consumed = 0;
	let match;

	while ((match = re.exec(text)) !== null) {
		if (match.index !== consumed && text.slice(consumed, match.index).trim() !== '') {
			throw new Error(`${context}: cannot parse attributes near "${text.slice(consumed)}".`);
		}
		const [full, name, doubleQuoted, singleQuoted] = match;
		const value = doubleQuoted ?? singleQuoted;
		if (value === undefined) {
			attrs.push(name);
		} else {
			if (value.includes('"')) {
				throw new Error(`${context}: attribute "${name}" contains a double quote.`);
			}
			attrs.push(`${name}="${value}"`);
		}
		consumed = match.index + full.length;
	}

	if (text.slice(consumed).trim() !== '') {
		throw new Error(`${context}: cannot parse attributes near "${text.slice(consumed)}".`);
	}

	return attrs.length ? ' ' + attrs.join(' ') : '';
}

/** Escape the characters MDX would read as expression or element syntax. */
function escapeProp(value) {
	return value.replace(/"/g, '&quot;').replace(/\{/g, '&#123;').replace(/</g, '&lt;');
}

/**
 * Split a region body at its shallowest heading level.
 *
 * The shallowest level is used rather than a fixed one so a source can nest
 * deeper headings inside a tab or a card without them becoming tabs of their
 * own.
 */
function splitAtHeadings(lines, context) {
	let level = Infinity;
	for (const line of lines) {
		if (line.fenced) continue;
		const heading = line.text.match(HEADING_RE);
		if (heading) level = Math.min(level, heading[1].length);
	}

	if (level === Infinity) {
		throw new Error(`${context}: region has no heading to split on.`);
	}

	const sections = [];
	let current = null;

	for (const line of lines) {
		const heading = line.fenced ? null : line.text.match(HEADING_RE);
		if (heading && heading[1].length === level) {
			current = { title: heading[2], attributes: '', body: [] };
			sections.push(current);
			continue;
		}
		if (current === null) {
			if (line.text.trim() !== '') {
				throw new Error(
					`${context}: text appears before the region's first heading — ` +
						`a split region may only contain headed sections.`
				);
			}
			continue;
		}

		// `<!--site:item …-->` carries the props a heading cannot: a card's
		// icon, say. It only counts directly under the heading, so it reads as
		// part of that heading rather than as something floating in the body.
		const item = line.fenced ? null : line.text.trim().match(ITEM_RE);
		if (item && current.body.every((text) => text.trim() === '')) {
			current.attributes = item[1];
			continue;
		}

		current.body.push(line.text);
	}

	return sections;
}

/** Trim leading and trailing blank lines without touching interior spacing. */
function trimBlankEdges(lines) {
	let start = 0;
	let end = lines.length;
	while (start < end && lines[start].trim() === '') start += 1;
	while (end > start && lines[end - 1].trim() === '') end -= 1;
	return lines.slice(start, end);
}

/**
 * Annotate every line with whether it sits inside a fenced code block.
 *
 * Directives are only recognised outside a fence, so a documented example of
 * the directive syntax stays an example.
 */
function markFences(markdown) {
	const out = [];
	let fence = null;

	for (const text of markdown.split('\n')) {
		const match = text.match(FENCE_RE);
		if (fence === null) {
			out.push({ text, fenced: false });
			if (match) fence = match[1];
		} else {
			const closes =
				match && match[1][0] === fence[0] && match[1].length >= fence.length && text.trim() === match[1];
			out.push({ text, fenced: true });
			if (closes) fence = null;
		}
	}

	return out;
}

function renderRegion(name, spec, attributes, body, context) {
	const outerAttrs = parseAttributes(attributes, context);

	if (spec.kind === 'wrap') {
		const inner = trimBlankEdges(body.map((line) => line.text));
		return [`<${spec.outer}${outerAttrs}>`, '', ...inner, '', `</${spec.outer}>`];
	}

	const sections = splitAtHeadings(body, context);
	const out = [`<${spec.outer}${outerAttrs}>`];

	for (const section of sections) {
		const itemAttrs = parseAttributes(section.attributes, `${context} (${section.title})`);
		out.push('', `<${spec.inner} ${spec.prop}="${escapeProp(section.title)}"${itemAttrs}>`, '');
		out.push(...trimBlankEdges(section.body));
		out.push('', `</${spec.inner}>`);
	}

	out.push('', `</${spec.outer}>`);
	return out;
}

/**
 * Replace every `<!--site:…-->` directive with its Starlight component.
 *
 * `context` names the source document, so a malformed directive reports which
 * file to fix rather than which line of the generator noticed.
 */
export function expandDirectives(markdown, context) {
	const lines = markFences(markdown);
	const out = [];
	const stack = [];

	for (let i = 0; i < lines.length; i += 1) {
		const line = lines[i];

		if (line.fenced) {
			(stack.length ? stack[stack.length - 1].body : out).push(line);
			continue;
		}

		const directive = line.text.trim().match(DIRECTIVE_RE);

		if (directive) {
			const [, closing, name, attributes] = directive;
			const spec = REGIONS[name];

			// `item` is not a region of its own: it annotates the heading above
			// it, so it travels to the enclosing region's body untouched and is
			// consumed there by splitAtHeadings.
			if (name === 'item' && !closing) {
				const open = stack[stack.length - 1];
				if (!open || REGIONS[open.name]?.kind !== 'split') {
					throw new Error(
						`${context}: <!--site:item--> is only valid directly under a heading ` +
							`inside a tabs or cards region.`
					);
				}
				open.body.push(line);
				continue;
			}

			if (!spec) {
				if (INLINE[name]) {
					throw new Error(
						`${context}: "${name}" is an inline directive and cannot open a region.`
					);
				}
				throw new Error(`${context}: unknown directive "site:${name}".`);
			}

			if (closing) {
				const open = stack.pop();
				if (!open || open.name !== name) {
					throw new Error(
						`${context}: <!--site:/${name}--> closes ${open ? `"${open.name}"` : 'nothing'}.`
					);
				}
				const rendered = renderRegion(
					open.name,
					spec,
					open.attributes,
					open.body,
					`${context}: site:${name}`
				);
				const target = stack.length ? stack[stack.length - 1].body : out;
				for (const text of rendered) target.push({ text, fenced: false });
				continue;
			}

			stack.push({ name, attributes, body: [] });
			continue;
		}

		(stack.length ? stack[stack.length - 1].body : out).push(line);
	}

	if (stack.length) {
		throw new Error(`${context}: <!--site:${stack[stack.length - 1].name}--> is never closed.`);
	}

	return out
		.map((line) => {
			if (line.fenced) return line.text;
			return line.text.replace(INLINE_DIRECTIVE_RE, (whole, name, attributes) => {
				const component = INLINE[name];
				if (!component) {
					if (REGIONS[name]) {
						throw new Error(`${context}: "${name}" is a region and needs its own line.`);
					}
					throw new Error(`${context}: unknown directive "site:${name}".`);
				}
				return `<${component}${parseAttributes(attributes, `${context}: site:${name}`)} />`;
			});
		})
		.join('\n');
}

/**
 * The import statement a page needs, or '' when it uses no components.
 *
 * Declared per entry in the manifest rather than inferred from the expanded
 * body: an import the page does not use is a build error worth having at the
 * manifest, where the page's shape is described.
 */
export function buildImports(names) {
	if (!names || names.length === 0) return '';
	const unique = [...new Set(names)].sort();
	return `import { ${unique.join(', ')} } from '@astrojs/starlight/components';\n\n`;
}

/** Every component name a document's expanded body actually references. */
export function usedComponents(markdown) {
	// A capitalised name inside a code span is text, not an element: the option
	// tables document `--config-file <PATH>`, which is not a component called
	// PATH. Fenced blocks are already stripped by the caller; inline spans are
	// invisible to it, so they are stripped here.
	const prose = markdown.replace(/(`+)[\s\S]*?\1/g, ' ');
	const found = new Set();
	for (const [, name] of prose.matchAll(/<([A-Z][A-Za-z0-9]*)[\s/>]/g)) found.add(name);
	return found;
}
