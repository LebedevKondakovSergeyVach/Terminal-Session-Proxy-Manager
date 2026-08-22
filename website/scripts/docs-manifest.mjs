// The single declaration of what the site is built from. Titles and
// descriptions live here rather than in the sources, because the sources are
// GitHub documents and are out of scope for this work.

/**
 * @typedef {object} PageEntry
 * @property {string}  source         Path relative to the repository root.
 * @property {string}  slug           Site slug, without locale prefix or slashes.
 * @property {'en'|'ru'} locale
 * @property {string}  dest           Path relative to `src/content/docs/`.
 * @property {string}  description    Used for `og:description` and search results.
 * @property {boolean} stripPreamble  Remove everything above the first H1.
 * @property {boolean} stripBadges    Remove status-badge-only lines.
 * @property {boolean} stripHeroImage Remove the repository banner image.
 */

/** @type {PageEntry[]} */
export const PAGES = [
	{
		source: 'README.md',
		slug: 'overview',
		locale: 'en',
		dest: 'overview.md',
		description: 'What Terminal Session Proxy Manager does and why it exists.',
		stripPreamble: true,
		stripBadges: true,
		stripHeroImage: true,
	},
	{
		source: 'README.ru.md',
		slug: 'overview',
		locale: 'ru',
		dest: 'ru/overview.md',
		description: 'Что такое Terminal Session Proxy Manager и зачем он нужен.',
		stripPreamble: true,
		stripBadges: true,
		stripHeroImage: true,
	},
	{
		source: 'docs/INSTALLATION.md',
		slug: 'installation',
		locale: 'en',
		dest: 'installation.md',
		description: 'Installing on macOS and Linux via Homebrew, Cargo or a prebuilt binary.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/INSTALLATION.ru.md',
		slug: 'installation',
		locale: 'ru',
		dest: 'ru/installation.md',
		description: 'Установка на macOS и Linux через Homebrew, Cargo или готовый бинарник.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/CONFIGURATION.md',
		slug: 'configuration',
		locale: 'en',
		dest: 'configuration.md',
		description: 'Profiles, settings and the precedence rules for config file resolution.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/CONFIGURATION.ru.md',
		slug: 'configuration',
		locale: 'ru',
		dest: 'ru/configuration.md',
		description: 'Профили, настройки и правила разрешения путей к конфигурации.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/USAGE.md',
		slug: 'usage',
		locale: 'en',
		dest: 'usage.md',
		description: 'Every subcommand, its flags and its exit codes.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/USAGE.ru.md',
		slug: 'usage',
		locale: 'ru',
		dest: 'ru/usage.md',
		description: 'Все подкоманды, их опции и коды возврата.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/SHELL_INTEGRATION.md',
		slug: 'shell-integration',
		locale: 'en',
		dest: 'shell-integration.md',
		description: 'Wiring the shell function into zsh and bash so the session environment changes.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'docs/SHELL_INTEGRATION.ru.md',
		slug: 'shell-integration',
		locale: 'ru',
		dest: 'ru/shell-integration.md',
		description: 'Подключение shell-функции в zsh и bash для смены окружения сессии.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'CONTRIBUTING.md',
		slug: 'contributing',
		locale: 'en',
		dest: 'contributing.md',
		description: 'Development setup, verification commands and the branching model.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'CHANGELOG.md',
		slug: 'changelog',
		locale: 'en',
		dest: 'changelog.md',
		description: 'Release history, following Keep a Changelog.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
	{
		source: 'CHANGELOG.ru.md',
		slug: 'changelog',
		locale: 'ru',
		dest: 'ru/changelog.md',
		description: 'История релизов в формате Keep a Changelog.',
		stripPreamble: false,
		stripBadges: false,
		stripHeroImage: false,
	},
];
