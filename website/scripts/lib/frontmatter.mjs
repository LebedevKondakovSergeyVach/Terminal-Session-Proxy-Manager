/**
 * Serialise a string as a YAML single-quoted scalar.
 *
 * The previous generator concatenated titles straight into the frontmatter, so
 * any title containing a colon produced an unparsable document.
 */
export function yamlString(value) {
	return `'${String(value).replace(/'/g, "''")}'`;
}

/**
 * Reject a field the manifest forgot to supply.
 *
 * `String(undefined)` is `'undefined'`, so a missing or misspelled
 * `description` would otherwise publish a page whose `og:description` and
 * search snippet read, literally, "undefined".
 */
function assertScalar(key, value) {
	if (typeof value !== 'string' || value.trim() === '') {
		throw new Error(`Frontmatter field "${key}" must be a non-empty string; got ${JSON.stringify(value)}.`);
	}
}

/** Build a frontmatter block from an ordered map of scalar fields. */
export function buildFrontmatter(fields) {
	const body = Object.entries(fields)
		.map(([key, value]) => {
			assertScalar(key, value);
			return `${key}: ${yamlString(value)}`;
		})
		.join('\n');
	return `---\n${body}\n---\n\n`;
}
