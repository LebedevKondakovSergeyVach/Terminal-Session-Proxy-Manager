// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import md3Theme from 'starlight-theme-md3';
import { viewTransitions } from 'astro-vtbot/starlight-view-transitions';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Terminal Session Proxy Manager',
			plugins: [
				viewTransitions(),
				md3Theme({
					accent: 'orange',
					shape: 'large',
					variant: 'fidelity',
					density: 'comfortable',
				}),
			],
			defaultLocale: 'root',
			locales: {
				root: {
					label: 'EN',
					lang: 'en',
				},
				ru: {
					label: 'RU',
					lang: 'ru',
				},
			},
			components: {
				ThemeSelect: './src/components/ThemeSelect.astro',
			},
			customCss: [
				'./src/styles/custom.css',
			],
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager' }],
			sidebar: [
				{
					label: 'Start Here',
					translations: { ru: 'Начало работы' },
					items: [
						{ label: 'Overview', translations: { ru: 'Обзор' }, slug: 'overview' },
						{ label: 'Installation', translations: { ru: 'Установка' }, slug: 'installation' },
						{ label: 'Contributing', translations: { ru: 'Разработка' }, slug: 'contributing' },
						{ label: 'Changelog', translations: { ru: 'История изменений' }, slug: 'changelog' },
					],
				},
				{
					label: 'Guides',
					translations: { ru: 'Руководства' },
					items: [
						{ label: 'Configuration', translations: { ru: 'Конфигурация' }, slug: 'configuration' },
						{ label: 'Usage', translations: { ru: 'Использование' }, slug: 'usage' },
						{ label: 'Shell Integration', translations: { ru: 'Интеграция с Shell' }, slug: 'shell-integration' },
					],
				},
			],
		}),
	],
});
