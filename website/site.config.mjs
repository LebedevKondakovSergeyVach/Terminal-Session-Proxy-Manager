// website/site.config.mjs
// Single definition of the deployment origin. Both the Astro config and the
// docs generator import this: the generator has to emit base-prefixed links
// itself, because Starlight only prefixes sidebar and site-title links.

/** Origin the site is served from. */
export const SITE = 'https://lebedevkondakovsergeyvach.github.io';

/** Sub-path of the GitHub Pages project page. No trailing slash. */
export const BASE = '/Terminal-Session-Proxy-Manager';

/** Prefix for links to repository files that have no page on the site. */
export const REPO_BLOB_URL =
	'https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/blob/main/';
