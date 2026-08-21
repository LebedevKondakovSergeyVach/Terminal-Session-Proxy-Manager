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
1. **🚨 MANDATORY - MCP Servers & Skills**: When making structural changes or fixing bugs, you MUST use the `astro-docs` MCP server to get the latest API references. If you need external tools or integrations, use `brave-search` or `github` MCP servers. ALWAYS rely on these tools instead of guessing.
2. **Material Design & View Transitions**: This site uses `starlight-theme-md3` (Material Design 3) and `<ClientRouter />` for SPA-like page transitions. When writing client-side `<script>` tags, remember to listen to `astro:page-load` because scripts don't re-run on soft navigations. 
3. **Markdown First**: Starlight converts `.md`/`.mdx` directly into styled UI pages. Do not build custom layout components for content that can be represented with standard markdown or Starlight's built-in components (`<Tabs>`, `<Card>`, `<Aside>`).
4. **i18n (Translations)**: The project maintains both English and Russian documentation. When adding a new page, ensure you configure it in both languages.
5. **Links**: Use relative links for internal documentation routing (e.g. `[Features](../features)`).
