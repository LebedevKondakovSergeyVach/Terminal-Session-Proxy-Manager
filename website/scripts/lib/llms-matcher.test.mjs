import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { test } from 'node:test';

// Resolve from the plugin so these checks exercise its overridden dependency.
const matcher = createRequire(import.meta.resolve('starlight-llms-txt'))('micromatch');
const pages = ['index', 'overview', 'shell-integration', 'ru/index', 'ru/overview', 'ru/shell-integration'];

test('the LLM matcher preserves the default homepage promotion', () => {
	assert.deepEqual(pages.filter(id => matcher.isMatch(id, ['index*'])), ['index']);
});

test('the LLM matcher accepts empty include and exclude lists', () => {
	assert.deepEqual(pages.filter(id => matcher.isMatch(id, [])), []);
	assert.deepEqual(pages.filter(id => !matcher.isMatch(id, [])), pages);
});

test('the LLM matcher selects localized pages with multiple glob patterns', () => {
	assert.deepEqual(
		pages.filter(id => matcher.isMatch(id, ['overview', 'ru/**'])),
		['overview', 'ru/index', 'ru/overview', 'ru/shell-integration'],
	);
	assert.deepEqual(
		pages.filter(id => !matcher.isMatch(id, ['**/index', 'shell-*'])),
		['overview', 'ru/overview', 'ru/shell-integration'],
	);
});

test('the LLM matcher preserves brace and extglob selectors', () => {
	for (const pattern of ['{overview,shell-integration}', '@(overview|shell-integration)']) {
		assert.deepEqual(pages.filter(id => matcher.isMatch(id, pattern)), ['overview', 'shell-integration']);
	}
});
