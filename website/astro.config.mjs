// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { viewTransitions } from 'astro-vtbot/starlight-view-transitions';
import { unified } from '@astrojs/markdown-remark';
import md3Theme from 'starlight-theme-md3';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightLlmsTxt from 'starlight-llms-txt';

import { SITE, BASE } from './site.config.mjs';

// https://astro.build/config
export default defineConfig({
	site: SITE,
	base: BASE,
	// Pinned explicitly rather than left to the default: the generator emits
	// absolute links with a trailing slash, and dev has to resolve them the
	// same way the static build on GitHub Pages does.
	trailingSlash: 'always',
	build: { format: 'directory' },
	// starlight-image-zoom injects a rehype plugin and does not yet support
	// Sätteri, Astro 7's default Markdown processor. Fall back to the
	// remark/rehype pipeline it does support.
	// https://github.com/HiDeoo/starlight-image-zoom/issues/63
	markdown: { processor: unified() },
	integrations: [
		starlight({
			title: 'Terminal Session Proxy Manager',
			plugins: [
				viewTransitions(),
				// Material Design 3 shapes, elevation and motion.
				//
				// `seed` generates the whole tonal palette from the project's own
				// rust orange — hsl(25 80% 50%), the accent this site already used.
				// The earlier attempt at this theme instead picked the canned
				// `accent: 'orange'` and then repainted the derived `--md3-comp-*`
				// tokens from custom.css with fifteen `!important` declarations.
				// Seeding is the supported way to get a specific brand colour, so
				// there is nothing left to out-rank. If a rule here ever seems to
				// need `!important`, reconfigure the theme instead — that is the
				// signal the two are fighting again.
				md3Theme({
					seed: '#E66E1A',
					// `content` keeps the derived palette faithful to the seed.
					// This matters because the selected sidebar item is painted
					// with `--md-sys-color-secondary-container`, and the variants
					// derive secondary very differently from the same orange:
					// expressive gives #006c46 (green), tonalSpot #765848,
					// content #7d5540 — a warm tan that stays in the orange family.
					variant: 'content',
					shape: 'large',
					density: 'comfortable',
					// State layers on hover, a ripple from the pointer, and brief
					// navigation feedback. On by default; named because it is the
					// reason the theme is here.
					motion: true,
				}),
				starlightImageZoom(),
				starlightLlmsTxt(),
				// A dead internal link fails the build. This is the gate that would
				// have caught the `href="CHANGELOG.md"` links shipped previously.
				starlightLinksValidator({ errorOnRelativeLinks: true }),
			],
			defaultLocale: 'root',
			locales: {
				root: { label: 'EN', lang: 'en' },
				ru: { label: 'RU', lang: 'ru' },
			},
			components: {
				ThemeSelect: './src/components/ThemeSelect.astro',
				Search: './src/components/Search.astro',
			},
			customCss: ['./src/styles/custom.css'],
			head: [
				{
					tag: 'meta',
					attrs: { property: 'og:image', content: `${SITE}${BASE}/og.jpg` },
				},
				{
					tag: 'meta',
					attrs: { name: 'twitter:image', content: `${SITE}${BASE}/og.jpg` },
				},
			],
			social: [
				{
					icon: 'github',
					label: 'GitHub',
					href: 'https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager',
				},
			],
			sidebar: [
				{
					label: 'Start Here',
					translations: { ru: 'Начало работы' },
					items: [
						{ label: 'Overview', translations: { ru: 'Обзор' }, slug: 'overview' },
						{ label: 'Installation', translations: { ru: 'Установка' }, slug: 'installation' },
						{ label: 'Changelog', translations: { ru: 'История изменений' }, slug: 'changelog' },
					],
				},
				{
					label: 'Guides',
					translations: { ru: 'Руководства' },
					items: [
						{ label: 'Configuration', translations: { ru: 'Конфигурация' }, slug: 'configuration' },
						{ label: 'Usage', translations: { ru: 'Использование' }, slug: 'usage' },
						{
							label: 'Shell Integration',
							translations: { ru: 'Интеграция с Shell' },
							slug: 'shell-integration',
						},
					],
				},
				{
					label: 'Project',
					translations: { ru: 'Проект' },
					items: [
						{
							label: 'Contributing',
							translations: { ru: 'Разработка' },
							slug: 'contributing',
							// No CONTRIBUTING.ru.md exists. Starlight falls back to the
							// English page rather than 404ing, so say so instead of
							// silently serving English under a Russian label.
							badge: { text: { en: 'EN', ru: 'на английском' }, variant: 'note' },
						},
					],
				},
			],
		}),
	],
});
