import { test } from 'node:test';
import assert from 'node:assert/strict';

import { yamlString, buildFrontmatter } from './frontmatter.mjs';

test('a plain string is single-quoted', () => {
	assert.equal(yamlString('Installation'), "'Installation'");
});

test('a colon in a title cannot break the document', () => {
	assert.equal(yamlString('Setup: macOS'), "'Setup: macOS'");
});

test('a single quote is doubled rather than escaped with a backslash', () => {
	assert.equal(yamlString("Don't"), "'Don''t'");
});

test('buildFrontmatter emits a delimited block with a trailing blank line', () => {
	assert.equal(
		buildFrontmatter({ title: 'A', description: 'B' }),
		"---\ntitle: 'A'\ndescription: 'B'\n---\n\n"
	);
});
