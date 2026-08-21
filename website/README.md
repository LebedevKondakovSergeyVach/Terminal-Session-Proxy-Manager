# Terminal Session Proxy Manager - Documentation Website

This is the Astro Starlight documentation website for `Terminal-Session-Proxy-Manager`.

## 🎨 Theme & Styling

This website uses a customized **Material Design 3** theme via `starlight-theme-md3`.

Key visual features:
- **Orange Accent:** The theme is configured with an expressive orange palette (`accent: 'orange'`, `variant: 'expressive'`).
- **Rounded Corners & Spacious Layout:** Set to `shape: 'large'` and `density: 'comfortable'`.
- **Astro View Transitions:** We use Astro's `<ClientRouter />` to enable SPA-like smooth fading transitions between pages without full browser reloads.
- **Circular Theme Toggle:** The dark/light mode toggle (`ThemeSelect.astro`) features a custom `document.startViewTransition()` implementation that creates a smooth expanding circle effect from the cursor when switched.
- **Typography:** Uses **Inter** for standard text and **JetBrains Mono** for code blocks, with customized Material Design scrollbars.

## 🚀 Project Structure

```text
website/
├── src/
│   ├── components/       # Custom overrides (ThemeSelect.astro, Head.astro)
│   ├── content/docs/     # Markdown (.md, .mdx) pages (English and Russian)
│   ├── styles/           # custom.css (fonts, scrollbars, overrides)
│   └── assets/           # Images and static assets
├── astro.config.mjs      # Starlight and md3Theme configuration
└── package.json
```

## 🧞 Commands

Run these from the `website/` directory:

| Command           | Action                                           |
| :---------------- | :----------------------------------------------- |
| `npm install`     | Installs dependencies                            |
| `npm run dev`     | Starts local dev server at `localhost:4321`      |
| `npm run build`   | Build your production site to `./dist/`          |
| `npm run preview` | Preview your build locally, before deploying     |

## 🤖 For AI Agents

Agents working on this website **MUST** read `AGENTS.md` and use the available MCP servers (like `astro-docs`) and skills (`starlight-website`).
