---
name: starlight-website
description: Use when working on the Astro Starlight documentation site in website/ — building it, adding or changing a page, editing the sidebar, styling, or debugging its content generator.
---

# Starlight website

1. Read [`website/AGENTS.md`](../../../website/AGENTS.md), the owning contract.
   Check the installed dependency versions in `website/package-lock.json`.
2. Locate the source: canonical Markdown for content, the manifest/sidebar for
   page routing, or website config/component/styles for UI. The project map's
   documentation flow identifies the generator stages. Edit source, never output.
3. Consult astro-docs first, then context7 or official documentation for changed
   APIs. Resolve context7 IDs when needed; do not keep an ID/version inventory.
4. Make the change in both locales. Keep the generator's source-shape,
   component and link checks; fix failures at their source.
5. Use `npm run sync` or `npm run dev` from `website/` while iterating. Check
   `npm run astro -- dev --help` for server options supported by the installed
   version rather than assuming background mode.
6. Run `verify-project`, including the website gate. For UI changes, inspect
   changed interactions in both languages, theme modes, keyboard and narrow/touch
   layouts; the build alone does not cover them. Report any omitted checks.

Deployment policy belongs to the website contract and `.ai/GIT_WORKFLOW.md`.
A documentation task does not authorize publishing.
