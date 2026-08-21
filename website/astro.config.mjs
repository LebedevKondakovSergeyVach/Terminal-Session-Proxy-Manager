// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import md3Theme from 'starlight-theme-md3';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Terminal Session Proxy Manager',
			plugins: [
				md3Theme({
					accent: 'orange',
					shape: 'large',
					variant: 'expressive',
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
				Head: './src/components/Head.astro',
			},
			customCss: [
				'./src/styles/custom.css',
			],
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager' }],
			sidebar: [
				{
					label: 'Guides',
					items: [
						// Each item here is one entry in the navigation menu.
						{ label: 'Example Guide', slug: 'guides/example' },
					],
				},
				{
					label: 'Reference',
					items: [{ autogenerate: { directory: 'reference' } }],
				},
			],
		}),
	],
});
