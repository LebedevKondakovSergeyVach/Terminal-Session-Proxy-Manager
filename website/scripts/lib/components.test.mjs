import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
	buildImports,
	expandDirectives,
	parseAttributes,
	usedComponents,
} from './components.mjs';

const SRC = 'docs/EXAMPLE.md';

test('a wrap region surrounds its body with the component', () => {
	const out = expandDirectives('<!--site:steps-->\n1. One\n2. Two\n<!--site:/steps-->', SRC);
	assert.equal(out, '<Steps>\n\n1. One\n2. Two\n\n</Steps>');
});

test('a wrap region keeps its attributes', () => {
	const out = expandDirectives('<!--site:aside type="tip"-->\nBe careful.\n<!--site:/aside-->', SRC);
	assert.equal(out, '<Aside type="tip">\n\nBe careful.\n\n</Aside>');
});

test('a tabs region turns each heading into a labelled tab and drops the heading', () => {
	const out = expandDirectives(
		['<!--site:tabs-->', '## Homebrew', '', 'brew install x', '', '## Cargo', '', 'cargo install x', '<!--site:/tabs-->'].join('\n'),
		SRC
	);
	assert.equal(
		out,
		[
			'<Tabs>',
			'',
			'<TabItem label="Homebrew">',
			'',
			'brew install x',
			'',
			'</TabItem>',
			'',
			'<TabItem label="Cargo">',
			'',
			'cargo install x',
			'',
			'</TabItem>',
			'',
			'</Tabs>',
		].join('\n')
	);
	assert.ok(!out.includes('## Homebrew'), 'the heading must not be printed twice');
});

test('a cards region uses the heading as the card title and keeps outer flags', () => {
	const out = expandDirectives(
		'<!--site:cards stagger-->\n### Session Control\n\n`proxy on`\n<!--site:/cards-->',
		SRC
	);
	assert.ok(out.startsWith('<CardGrid stagger>'));
	assert.ok(out.includes('<Card title="Session Control">'));
	assert.ok(out.includes('`proxy on`'));
});

test('a split region splits on its shallowest heading level only', () => {
	const out = expandDirectives(
		'<!--site:tabs-->\n## One\n\n### Nested\n\nbody\n\n## Two\n\nx\n<!--site:/tabs-->',
		SRC
	);
	assert.equal(out.match(/<TabItem/g).length, 2);
	assert.ok(out.includes('### Nested'), 'a deeper heading stays a heading');
});

test('directives inside a fenced code block are left alone', () => {
	const source = ['```md', '<!--site:steps-->', '1. One', '<!--site:/steps-->', '```'].join('\n');
	assert.equal(expandDirectives(source, SRC), source);
});

test('a heading inside a fence does not split a region', () => {
	const out = expandDirectives(
		['<!--site:tabs-->', '## Real', '', '```bash', '# not a heading', '```', '<!--site:/tabs-->'].join('\n'),
		SRC
	);
	assert.equal(out.match(/<TabItem/g).length, 1);
	assert.ok(out.includes('# not a heading'));
});

test('regions nest', () => {
	const out = expandDirectives(
		'<!--site:tabs-->\n## One\n\n<!--site:steps-->\n1. a\n<!--site:/steps-->\n<!--site:/tabs-->',
		SRC
	);
	assert.ok(out.includes('<TabItem label="One">'));
	assert.ok(out.includes('<Steps>'));
	assert.ok(out.indexOf('<Steps>') > out.indexOf('<TabItem'));
});

test('an inline directive is replaced in place', () => {
	const out = expandDirectives('## What you get <!--site:badge text="Core" variant="tip"-->', SRC);
	assert.equal(out, '## What you get <Badge text="Core" variant="tip" />');
});

test('a title containing quotes or braces is escaped for MDX', () => {
	const out = expandDirectives('<!--site:cards-->\n### A {b} <c>\n\nx\n<!--site:/cards-->', SRC);
	assert.ok(out.includes('<Card title="A &#123;b&#125; &lt;c>">') === false);
	assert.ok(out.includes('&#123;b}'), 'an opening brace must not reach MDX raw');
	assert.ok(out.includes('&lt;c>'), 'an opening angle bracket must not reach MDX raw');
});

test('an unclosed region throws and names the source', () => {
	assert.throws(() => expandDirectives('<!--site:steps-->\n1. One', SRC), /EXAMPLE\.md.*never closed/s);
});

test('a mismatched closer throws', () => {
	assert.throws(
		() => expandDirectives('<!--site:steps-->\nx\n<!--site:/tabs-->', SRC),
		/closes "steps"/
	);
});

test('an unknown directive throws rather than passing through', () => {
	assert.throws(() => expandDirectives('<!--site:carousel-->\nx\n<!--site:/carousel-->', SRC), /unknown directive/);
});

test('a split region with no heading throws', () => {
	assert.throws(() => expandDirectives('<!--site:tabs-->\njust text\n<!--site:/tabs-->', SRC), /no heading/);
});

test('text before a split region’s first heading throws', () => {
	assert.throws(
		() => expandDirectives('<!--site:tabs-->\nstray\n\n## One\n\nx\n<!--site:/tabs-->', SRC),
		/before the region's first heading/
	);
});

test('an inline directive used as a region throws', () => {
	assert.throws(() => expandDirectives('<!--site:badge text="x"-->\ny\n<!--site:/badge-->', SRC), /inline directive/);
});

test('a document with no directives is returned unchanged', () => {
	const source = '# Title\n\nSome prose with <!-- an ordinary comment -->.\n';
	assert.equal(expandDirectives(source, SRC), source);
});

test('parseAttributes accepts bare flags and quoted values', () => {
	assert.equal(parseAttributes('stagger', SRC), ' stagger');
	assert.equal(parseAttributes('type="tip"', SRC), ' type="tip"');
	assert.equal(parseAttributes("type='tip'", SRC), ' type="tip"');
	assert.equal(parseAttributes('', SRC), '');
});

test('parseAttributes rejects a value carrying a double quote', () => {
	assert.throws(() => parseAttributes(`text='a"b'`, SRC), /double quote/);
});

test('buildImports emits one sorted, deduplicated statement', () => {
	assert.equal(
		buildImports(['TabItem', 'Tabs', 'TabItem']),
		"import { TabItem, Tabs } from '@astrojs/starlight/components';\n\n"
	);
	assert.equal(buildImports([]), '');
	assert.equal(buildImports(undefined), '');
});

test('usedComponents finds every capitalised element', () => {
	const found = usedComponents('<Tabs>\n<TabItem label="x">y</TabItem>\n</Tabs>\n<Badge text="z" />');
	assert.deepEqual([...found].sort(), ['Badge', 'TabItem', 'Tabs']);
	assert.ok(!usedComponents('<div>x</div>').has('div'));
});

test('usedComponents ignores a capitalised name inside a code span', () => {
	const found = usedComponents('| `--config-file <PATH>` | `TSPM_CONFIG` |\n\n<Badge text="z" />');
	assert.deepEqual([...found].sort(), ['Badge']);
});

test('site:item supplies props the heading cannot carry', () => {
	const out = expandDirectives(
		'<!--site:cards stagger-->\n### Session Control\n<!--site:item icon="laptop"-->\n\n`proxy on`\n<!--site:/cards-->',
		SRC
	);
	assert.ok(out.includes('<Card title="Session Control" icon="laptop">'));
	assert.ok(!out.includes('site:item'), 'the directive itself must not survive');
});

test('site:item outside a split region throws', () => {
	assert.throws(
		() => expandDirectives('<!--site:steps-->\n<!--site:item icon="x"-->\n1. a\n<!--site:/steps-->', SRC),
		/only valid directly under a heading/
	);
	assert.throws(() => expandDirectives('<!--site:item icon="x"-->', SRC), /only valid directly under a heading/);
});

test('site:item away from its heading throws rather than reaching the page', () => {
	// It is only consumed directly under a heading. Anywhere else it survives
	// to the inline pass, which recognises no such component — so a misplaced
	// item fails the build instead of shipping a stray comment.
	assert.throws(
		() =>
			expandDirectives('<!--site:cards-->\n### A\n\nprose\n\n<!--site:item icon="x"-->\n<!--site:/cards-->', SRC),
		/unknown directive "site:item"/
	);
});
