## Website Development for AI Agents

When starting the dev server, use background mode:
```bash
astro dev --background
```
Manage the background server with `astro dev stop`, `astro dev status`, and `astro dev logs`.

## 🚨 MANDATORY: Skills & MCP Servers

When working on this website (or the main project), you **MUST** actively use the available Skills and MCP servers for any tasks or fixes. Do not guess APIs or write boilerplate from memory.

1. **MCP Servers:**
   - **`astro-docs`**: ALWAYS query this server (using `search_astro_docs`) when you need to use Astro APIs, configure View Transitions, or build Starlight components. The Astro ecosystem changes rapidly.
   - **`github`**: Use this when instructed to create PRs, read issues, or analyze the upstream project repository.
   - **`brave-search`**: Use `brave_web_search` to look up third-party Astro integrations, npm packages (like `starlight-theme-md3`), or community themes.

2. **Project Skills (`.agents/skills/`):**
   - **`starlight-website`**: Read this skill (`.agents/skills/starlight-website/SKILL.md`) for baseline rules about i18n, markdown usage, and directory structure.
   - **`verify-project`**: Use this skill to check CI formatting and lints after making global changes.

## Architecture Guidelines
- **Starlight + md3Theme:** We use `starlight-theme-md3` for Material Design. DO NOT manually strip out its classes. If you need to tweak the design, do it in `astro.config.mjs` (theme options) or `src/styles/custom.css`.
- **View Transitions:** We use `<ClientRouter />` for SPA-like transitions. Remember that standard `<script>` tags might not re-run on page transitions unless they listen to `astro:page-load`.
- **Components:** We have custom Starlight component overrides in `src/components/` (like `ThemeSelect.astro` with circular View Transition animation and `Head.astro`). Modify them carefully to avoid breaking animations.
- **i18n:** All content must be maintained in both English (`en/`) and Russian (`ru/`).
