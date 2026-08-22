/**
 * Serialise a string as a YAML single-quoted scalar.
 *
 * The previous generator concatenated titles straight into the frontmatter, so
 * any title containing a colon produced an unparsable document.
 */
export function yamlString(value) {
	return `'${String(value).replace(/'/g, "''")}'`;
}

/** Build a frontmatter block from an ordered map of scalar fields. */
export function buildFrontmatter(fields) {
	const body = Object.entries(fields)
		.map(([key, value]) => `${key}: ${yamlString(value)}`)
		.join('\n');
	return `---\n${body}\n---\n\n`;
}
