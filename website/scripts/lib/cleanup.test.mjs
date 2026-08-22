import { test } from 'node:test';
import assert from 'node:assert/strict';

import { extractTitle, applyCleanup } from './cleanup.mjs';

const README = [
	'<p align="center">',
	'  🇬🇧 <b>English</b> | 🇷🇺 <a href="README.ru.md">Русский</a>',
	'</p>',
	'',
	'# ⚡ Terminal Session Proxy Manager',
	'',
	'![Project Banner](assets/banner_new.jpg)',
	'',
	'[![CI](https://github.com/o/r/actions/workflows/ci.yml/badge.svg)](https://github.com/o/r/actions)',
	'[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)',
	'![Views](https://komarev.com/ghpvc/?username=x&label=views)',
	'',
	'Manage proxy profiles.',
	'',
].join('\n');

const entry = {
	source: 'README.md',
	stripPreamble: true,
	stripBadges: true,
	stripHeroImage: true,
};

test('extractTitle returns the first H1 and removes it from the body', () => {
	const { title, body } = extractTitle('# ⚡ Title\n\nProse.\n');
	assert.equal(title, '⚡ Title');
	assert.equal(body.includes('# ⚡ Title'), false);
	assert.equal(body.includes('Prose.'), true);
});

test('extractTitle throws when no H1 appears near the top', () => {
	assert.throws(() => extractTitle('\n\nJust prose, no heading.\n'), /H1/);
});

test('the HTML preamble above the H1 is removed', () => {
	const out = applyCleanup(README, entry);
	assert.equal(out.includes('align="center"'), false);
});

test('the banner image is removed', () => {
	const out = applyCleanup(README, entry);
	assert.equal(out.includes('banner_new.jpg'), false);
});

test('badge lines are removed but prose survives', () => {
	const out = applyCleanup(README, entry);
	assert.equal(out.includes('img.shields.io'), false);
	assert.equal(out.includes('komarev.com'), false);
	assert.equal(out.includes('badge.svg'), false);
	assert.equal(out.includes('Manage proxy profiles.'), true);
});

test('a content image survives while a badge on the same document is removed', () => {
	const mixed = [
		'# T',
		'',
		'[![Rust](https://img.shields.io/badge/rust-1.88-orange.svg)](https://www.rust-lang.org)',
		'',
		'![Dashboard](assets/proxy_dashboard_final.png)',
		'',
		'Prose.',
	].join('\n');
	const out = applyCleanup(mixed, {
		source: 'docs/USAGE.md',
		stripPreamble: false,
		stripBadges: true,
		stripHeroImage: false,
	});
	assert.equal(out.includes('img.shields.io'), false);
	assert.equal(out.includes('proxy_dashboard_final.png'), true);
});

test('stripBadges throws when the flag is set but the document has no badges', () => {
	assert.throws(
		() =>
			applyCleanup('# T\n\nProse only.\n', {
				source: 'README.md',
				stripPreamble: false,
				stripBadges: true,
				stripHeroImage: false,
			}),
		/badge/i
	);
});
