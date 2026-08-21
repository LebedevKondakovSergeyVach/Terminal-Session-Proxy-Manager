---
name: starlight-website
description: Instructions for managing, building, and expanding the Starlight (Astro) documentation website.
---

# Starlight Website Management

This skill provides guidelines and commands for developing the Astro Starlight documentation website for the `Terminal-Session-Proxy-Manager` project.

## Directory Structure
The website is located in the `website/` directory at the root of the repository.
- `website/src/content/docs/`: Where all the markdown (`.md`, `.mdx`) pages live.
- `website/astro.config.mjs`: The Starlight configuration (sidebar, i18n, title).
- `website/public/`: Static assets like images and favicons.

## Commands

Always run these commands from inside the `website/` directory!

```bash
cd website
npm run dev      # Start the local development server at http://localhost:4321
npm run build    # Build the static site into the dist/ directory (always run this before a PR)
npm run preview  # Preview the built static site locally
```

## Guidelines for Agents
1. **MCP Server**: When making structural changes or writing complex Astro components, query the `astro-docs` MCP server. The Astro ecosystem moves fast, and the MCP server has the latest up-to-date API references.
2. **Markdown First**: Starlight is designed to convert `.md` and `.mdx` directly into styled UI pages. Do not build custom React/Astro layout components for content that can be represented with standard markdown tables, headings, or Starlight's built-in components (like `<Tabs>`, `<Card>`, `<Aside>`).
3. **i18n (Translations)**: The project maintains both English and Russian documentation. When adding a new page, ensure you configure it in both languages according to Starlight's i18n structure.
4. **Links**: Use relative links for internal documentation routing (e.g. `[Features](../features)`).
