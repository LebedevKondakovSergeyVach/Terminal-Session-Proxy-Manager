import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';

export const collections = {
	docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
	// Starlight collects every `pagefind.*` key from this collection and hands
	// it to `new PagefindUI({ translations })`. Without it, Pagefind's UI stays
	// English on the Russian pages — which the search component used to paper
	// over by regex-rewriting the rendered message.
	//
	// `tspm.*` keys are the site's own UI strings, read with `Astro.locals.t`
	// like Starlight's, so a component never branches on the locale itself.
	i18n: defineCollection({
		loader: i18nLoader(),
		schema: i18nSchema({ extend: z.object({ 'tspm.home': z.string() }).partial() }),
	}),
};
