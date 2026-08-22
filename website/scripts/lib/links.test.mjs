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

test('a link inside a fenced code block is left unchanged', () => {
	const markdown = ['```bash', 'See [guide](docs/USAGE.md) for details.', '```', ''].join('\n');
	const out = rewriteLinks(markdown, { source: 'README.md', dest: 'overview.md' }, pageMap);
	assert.equal(out, markdown);
});

test('an unknown link target inside a fenced code block does not throw', () => {
	const markdown = ['```', '[nope](NOPE.md)', '```', ''].join('\n');
	assert.doesNotThrow(() => rewriteLinks(markdown, { source: 'README.md', dest: 'overview.md' }, pageMap));
});

test('a link inside an inline code span is left unchanged', () => {
	const markdown = 'Run `[nope](NOPE.md)` as shown.';
	const out = rewriteLinks(markdown, { source: 'README.md', dest: 'overview.md' }, pageMap);
	assert.equal(out, markdown);
});

test('a link whose display text is itself backtick-wrapped still has its href rewritten', () => {
	const markdown = '[`configs/config.default.json`](../configs/config.default.json)';
	const out = rewriteLinks(markdown, { source: 'docs/CONFIGURATION.md', dest: 'configuration.md' }, pageMap);
	assert.equal(
		out,
		'[`configs/config.default.json`](https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/blob/main/configs/config.default.json)'
	);
});

test('a link wrapping an image rewrites both the image path and the outer href', () => {
	const out = rewriteLinks(
		'[![alt](assets/proxy_dashboard_final.png)](docs/USAGE.md)',
		{ source: 'README.md', dest: 'overview.md' },
		pageMap
	);
	assert.equal(
		out,
		'[![alt](../../assets/proxy_dashboard_final.png)](/Terminal-Session-Proxy-Manager/usage/)'
	);
});
