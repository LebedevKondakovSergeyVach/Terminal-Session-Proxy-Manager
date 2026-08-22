// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { viewTransitions } from 'astro-vtbot/starlight-view-transitions';

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
	integrations: [
		starlight({
			title: 'Terminal Session Proxy Manager',
			plugins: [viewTransitions()],
			defaultLocale: 'root',
			locales: {
				root: { label: 'EN', lang: 'en' },
				ru: { label: 'RU', lang: 'ru' },
			},
			components: {
				ThemeSelect: './src/components/ThemeSelect.astro',
			},
			customCss: ['./src/styles/custom.css'],
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
