# Documentation diagrams — research and scope

Research date: 2026-10-09. Work targets `release/2.3.1` through the task
branch `docs/documentation-diagrams`. This opens the next release; it does
not cut or publish version 2.3.1.

## Reader problem

The useful distinction is between the profile saved in configuration, the
environment of the current shell, and the environment of a child command.
A diagram should make one of these boundaries visible rather than repeat a
command table.

Three independent read-only investigations covered reader needs, source-code
accuracy and rendering. Their important findings were checked against the
sources below before choosing this scope.

## Reference and current state

[AgentCommons](https://github.com/DrishtantKaushal/AgentCommons#architecture)
combines a text architecture diagram with Mermaid sequence and state diagrams.
The transferable idea is to explain a specific relationship or sequence in
each diagram. All diagrams for this project will be original.

The shell guide already has a Mermaid sequence diagram in both languages
(`docs/SHELL_INTEGRATION.md`, lines 43–54, and its Russian twin, lines 46–57,
before this change). It incorrectly shows `proxy switch` directly returning
exports. The actual wrapper first selects a profile and then calls `env on`
only on success and with a nonempty `ALL_PROXY` (`src/cmd/init.rs`, lines
16–29 and 100–113).

The existing site artifact was also inspected: `website/dist/shell-integration/index.html`
contained `<pre data-language="mermaid"><code>` at line 1098. The current site
therefore displayed Mermaid source as code. The configuration uses `unified()`
without a Mermaid integration (`website/astro.config.mjs`, line 31), and the
generator preserves fenced code (`website/scripts/lib/links.mjs`, lines 285–302).

## Recommended scope

| Document | Diagram and question | Evidence |
| :--- | :--- | :--- |
| Both READMEs, after the first-run example | Active profile branches into `proxy on` for the current shell and `proxy run -- <cmd>` for a child command. Where do the settings apply? | `src/proxy_env.rs`, lines 37–57; `src/main.rs`, lines 287–303 |
| Both shell guides, replacing the existing diagram | `proxy on` calls the binary's `env on`, receives quoted exports, and evaluates them in the shell. Who changes the environment? | `src/cmd/init.rs`, lines 14–15 and 98–99; `src/cmd/env.rs`, lines 19–28; `src/proxy_env.rs`, lines 74–79 |
| Both shell guides, in the dashboard section | Enter saves the profile, writes the hand-off file, and exits; the shell evaluates and removes the file. When does the environment change? | `src/cmd/dash.rs`, lines 148–154 and 428–443; `src/shell_handoff.rs`, lines 15 and 69–74; `src/cmd/init.rs`, lines 31–39 |

Keep ordinary prose alongside the diagrams. Explain that applications must
read the proxy environment, other existing terminals are unaffected, and
`profile use` saves selection without itself evaluating exports. Dashboard
Space saves selection but does not hand exports to the shell.

Do not add diagrams to installation or configuration: their linear commands
and precedence lists already answer the reader's question. Contributing
already has a project tree and branch diagram. A monitor failover diagram is
a possible future addition, but is outside this small scope. Agent instruction
entry points do not need user-facing diagrams.

## Format comparison

| Format | Benefit | Cost |
| :--- | :--- | :--- |
| Fenced `text` with box-drawing characters | Same readable structure on GitHub and the current site; editable in Markdown | Static; keep labels short and provide prose for accessibility |
| Mermaid | Native diagram rendering on GitHub | Needs site integration and verification in both Markdown and MDX |
| SVG images | Consistent graphic on both surfaces | Separate source/assets and generator image-path handling |

The initial recommendation was text diagrams for their minimal integration
cost. The maintainer chose **Mermaid plus site support** after this comparison.
Use a top-down flowchart for scope and sequence diagrams for the two shell
operations. Keep translated `accTitle` and `accDescr` in every diagram and
ordinary explanatory prose next to it.

The selected integration is `astro-mermaid` 2.1.0 with Mermaid 11, listed in
the [Starlight integration catalog](https://starlight.astro.build/resources/plugins/).
The npm metadata was checked: its peers accept Astro >=4 and Mermaid 10 or 11.
Mermaid 11 is explicitly selected rather than the incompatible Mermaid 12
latest version. This integration preserves the unified pipeline and renders
both Markdown and MDX fences in the browser.

Place it before Starlight so it transforms Mermaid fences before Expressive
Code. Browser verification is necessary because the static build does not
parse diagram syntax. Check both languages, light/dark/auto, a narrow viewport,
SVG accessibility descriptions and ordinary code blocks. Use inherited MD3
colour and shape roles in the supported `themeCSS`, with `theme: 'base'` and
`autoTheme: false`. Colours then follow `data-theme` without replacing the SVG
while the reader zooms. The site's Inter font is also used for diagram labels.
The diagram frame uses card roles, and the viewer uses dialog/control roles.
Loading backgrounds follow the same palette without a separate shimmer.

Keep inline labels at natural size; only diagrams wider than their container
scroll. The translated Expand control opens the original SVG in a native
dialog at 100%, with zoom in/out, Fit and Actual size. Drag, pinch, wheel and
keyboard controls operate inside its viewport. Fit exposes all four edges;
zoom stops between the fit scale and 400%. Preserve browser zoom shortcuts,
keep Tab/Shift+Tab inside the viewer, and restore SVG, scroll and opener focus
on close. Moving one SVG preserves its IDs, arrow markers and descriptions.

The MarkdownContent override composes the image-zoom plugin's public override
with the diagram viewer, preserving existing screenshot zoom. Hidden viewer UI
is excluded from Pagefind. Check 320/375/768/1440px with touch emulation on
the first three, both locales, all theme modes, reduced motion and rotation.

The alternatives were also checked: `@pasqal-io/starlight-client-mermaid` pins
the older Markdown processor 6 peer range, while ours is 7;
`rehype-mermaid` server rendering would add browser installation to CI. Neither
is needed for this scope.

Integration primary source: [astro-mermaid](https://github.com/joesaby/astro-mermaid).

Official references checked:

- [GitHub diagram support](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/creating-diagrams).
- [Astro Markdown](https://docs.astro.build/en/guides/markdown-content/).
- [Starlight Markdown authoring](https://starlight.astro.build/guides/authoring-content/).
- [Mermaid theme configuration](https://mermaid.js.org/config/theming.html).
- [Native modal dialogs](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement/showModal).
- [Pointer Events](https://www.w3.org/TR/pointerevents4/).

Context7 was accessed using the supplied key in an HTTP header, and the official
Starlight library ID was resolved as `/withastro/starlight`; the follow-up
viewer research resolved `/mermaid-js/mermaid` and queried its theme/CSS API.
The `astro-docs`
MCP endpoint returned HTTP 403 / Cloudflare 1010, so official web pages were
used as its fallback. No credentials belong in tracked files.

## Work checklist

- [x] Read SESSION, Claude Code instructions and applicable project skills.
- [x] Research with three independent read-only subagents and spot-check findings.
- [x] Update main, create and push `release/2.3.1`, create the task branch.
- [x] Add the three diagrams in English and Russian and correct adjacent prose.
- [x] Add Mermaid site support, theme handling and reduced-motion styling.
- [x] Match MD3 styling and add accessible zoom, pan and pinch controls.
- [x] Verify touch viewports, rotation, keyboard and existing image zoom.
- [x] Independently review the change using the installed review instructions.
- [x] Run the crate gate, website gate, agent-docs audit and inspect built pages.
- [x] Open a draft task PR against `release/2.3.1` with local validation results.
- [ ] Resolve the website CI dependency-audit blocker before marking the PR ready.

## Validation snapshot

All checks below were run on 2026-10-09. No Rust or CLI behavior changed.

| Check | Observed result | Exit |
| :--- | :--- | :--- |
| `cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked` | 94 library, 11 binary and 37 integration tests passed; 0 failed | 0 |
| `cd website && npm ci && npm test && npm run build` | `pass 55`, `fail 0`; `All internal links are valid`; `18 page(s) built`; `Complete!` | 0 |
| `bash .claude/skills/agent-docs-audit/audit.sh` | `No blocking problems.` | 0 |
| `git diff --check` | No output | 0 |
| Browser smoke of the built site | `cases: 24`, `passed: 24`, `failed: 0`; six SVG diagrams across EN/RU, widths 320/375/768/1440, light/dark/auto, touch pan/pinch, wheel, zoom/fit/actual, keyboard, focus, rotation, SVG descriptions, reduced motion and ordinary code blocks; no browser errors or external runtime requests | 0 |
| Additional browser regression | EN/RU screenshot zoom opens/closes; diagram zoom reaches 400% and stops at fit with disabled limit buttons | 0 |

Independent source review found interception of Ctrl/Cmd browser zoom shortcuts;
the modifier guard was added and verified. Independent visual review of desktop
and mobile screenshots found readable labels and complete controls at 320px.
The browser smoke verified 44px touch controls after the theme's opening
animation, fit containment, finite gesture geometry, stable SVG IDs through
theme changes, and page overflow prevention. Rotation changes page layout and
can clamp its scroll position before close; closing preserves that new position.

The preview was opened in Safari. Automated DOM inspection there was blocked
by its existing JavaScript-from-Apple-Events setting; the browser smoke used
Chromium instead. The MSRV toolchain and actual GitHub Mermaid renderer were
not run locally; MSRV subsequently passed in CI. The diagrams use the documented
basic flowchart/sequence and accessibility syntax.

The static build warns about a Mermaid chunk over 500 kB; Mermaid is loaded
only on pages with diagrams. The existing `/404` route warning also remains.

`npm audit --json --omit=dev` exits 1: the baseline lockfile has 22 findings
(14 moderate, 8 high); the changed lockfile has the same findings plus two low
findings for Mermaid/KaTeX. The new advisory is
[KaTeX trust restrictions after existing prototype pollution](https://github.com/advisories/GHSA-238p-pmpm-9mq7).
Mermaid 11 requests KaTeX `^0.16.47`; the advisory's fixed KaTeX 0.18.2 is
outside that range. A forced unrelated dependency upgrade was not applied.
The mandatory `npm audit --audit-level=high` step in
`.github/workflows/website.yml` fails on these existing high findings. The
generator tests and production build passed in that same run. All other
required PR checks passed, including both OS test jobs, MSRV and Rust security
advisories. [Draft PR #31](https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/pull/31)
remains blocked by the [website security step](https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/actions/runs/37903157157/job/113730200804).

A compatible lockfile refresh was tested only in a temporary baseline copy:
it reduced the findings to 13 moderate and 3 high, but the high
`braces` → `micromatch` → `starlight-llms-txt` chain remained. The
[braces advisory](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm)
lists no patched version; the registry's latest is still 3.0.3, and the latest
`starlight-llms-txt` still depends on the affected matcher. No forced downgrade,
audit suppression or unrelated dependency repair was applied to this task.

Docs-only changes do not require a changelog entry. Do not bump Cargo versions,
create tags, merge into main or deploy from these branches. Keep this research
in `.ai/plans/` until the work ships.
