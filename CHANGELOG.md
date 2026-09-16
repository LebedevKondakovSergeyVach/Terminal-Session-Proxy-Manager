# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.3.0] — 2026-09-16

The release that gives the project a home on the web. A bilingual documentation
site is published from this repository's own Markdown and works on phones as
well as desktops. The CLI's output is reworked for terminals and scripts that
do not want decoration, and a TLS advisory is fixed.

**Upgrading?** Three output changes can affect scripts that read what the tool
prints — they are marked **Breaking for scripts** under **Changed**. Nothing
changes in `config.json`, `settings.json`, the command names, flags or exit
codes.

### Security

- **CLI**: `rustls` updated to 0.23.45 for
  [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285.html):
  TLS 1.3 handshake messages sent across an encryption-level boundary were
  accepted instead of rejected. `reqwest` uses it for every HTTPS request the
  tool makes — `status`, `ping`, `speedtest`, `monitor`, imports from a URL.
- **Website**: The site's build dependencies are updated past advisories in
  `astro` (critical: remote code execution through AVIF image optimisation),
  `sharp`, `svgo` and `js-yaml`. Astro moves to 7.3.2. These affect only the
  site build, not the CLI.

### Added

#### Documentation website

- **Website**: A documentation site at
  [lebedevkondakovsergeyvach.github.io/Terminal-Session-Proxy-Manager](https://lebedevkondakovsergeyvach.github.io/Terminal-Session-Proxy-Manager/),
  in English and Russian (`/ru/`), built with Astro Starlight. Its pages are
  generated from `README*.md`, `docs/*.md`, `CONTRIBUTING.md` and the
  changelogs at build time, so the site and the files on GitHub cannot drift
  apart.
- **Website**: A home page with the project's features side by side with
  terminal screenshots in light and dark variants, and quick-start tiles.
- **Website**: Material Design 3 styling whose whole palette is derived from
  the project's rust orange, in light and dark, with smooth transitions between
  pages that fall back to a plain swap when reduced motion is requested.
- **Website**: A three-way theme switch — light, auto (follows the system) and
  dark — with a circular reveal when the theme changes.
- **Website**: Full-text search, with quick links to the main guides while the
  search box is empty, and a search interface translated into Russian.
- **Website**: Install instructions in tabs (Homebrew, Cargo, prebuilt binary),
  step-by-step guides, colour-coded callouts, terminal-framed code blocks with
  a copy button, and screenshots that zoom on click or tap.
- **Website**: Works on phones and touch screens:
  - wide tables scroll sideways instead of being cut off;
  - tab labels wrap on narrow screens instead of running off the edge;
  - buttons, tabs and selects are at least 44px tall to tap;
  - hover effects apply only with a mouse, so a tapped card no longer stays
    raised;
  - the header fits a 320px-wide screen;
  - on phones the theme and language controls of a docs page live in the
    menu.
- **Website**: A round home button beside the site title on every docs page, on
  phones and desktops, labelled "Home" / "На главную" for screen readers.
- **Website**: The project's logo in the header of the home page, its own
  favicon, and an icon for home-screen bookmarks on iOS.
- **Website**: A custom 404 page in both languages, with links back into the
  documentation and a way home on every screen width.
- **Website**: A preview image for links shared on social networks and in
  messengers, and `llms.txt` for machine readers.
- **Website**: The build fails on a broken internal link, so a dead link cannot
  reach the published site.

#### Documentation

- **Docs**: `CHANGELOG.ru.md` — the release history in Russian, published at
  `/ru/changelog/`. `README.ru.md` now links to it rather than to the English
  file.
- **Docs**: `docs/SHELL_INTEGRATION.md` explains *why* the shell function is
  needed, with a sequence diagram; `docs/USAGE.md` opens with the subcommands
  grouped into four categories.
- **Docs**: The READMEs carry a badge linking to the documentation site.

#### CLI and configuration

- **Config**: OpenAI (`https://status.openai.com/api/v2/status.json`) and
  Anthropic (`https://www.anthropic.com`) joined the default `ping_targets`.
  Existing configs keep their own list.
- **CLI**: After switching, `proxy switch` and `proxy use <key>` end the
  confirmation line with `[ON]` or `[OFF]` — whether proxy variables are set in
  this shell. Through the `proxy` shell function, `[ON]` means the new profile
  has just been applied; `[OFF]` means it is saved and waits for `proxy on`.
- **CLI**: The `proxy switch` menu is cleared from the screen when it closes,
  whether you chose a profile or cancelled.
- **CLI**: `proxy ping` prints HTTP 2xx responses in green and everything else
  in yellow.

#### Automation and contributing

- **CI**: `website.yml` gates every pull request that touches the site or the
  documents it is built from — it runs the generator's tests, builds the site
  and fails on a dead internal link. `pages.yml` publishes the site from
  `main`.
- **CI**: Dependabot watches the website's npm dependencies as well as Cargo
  and the GitHub Actions.
- **CI**: The branch policy requires `CHANGELOG.ru.md` to change whenever
  `CHANGELOG.md` does, since both are published.
- **Contributing**: Git hooks in `.githooks/`, enabled once per clone with
  `git config core.hooksPath .githooks`. `commit-msg` rejects tool attribution
  lines (`Co-Authored-By:`, "Generated with …"); `pre-push` refuses pushes to
  `main` and tag pushes, since merging a release branch is what publishes.
- **Contributing**: One set of instructions for AI coding agents. `AGENTS.md`
  is the shared contract; `CLAUDE.md`, `GEMINI.md` and `.cursor/rules/` are the
  entry points for Claude Code, Antigravity and Cursor; area rules live in
  `.claude/rules/`. The task skills in `.claude/skills/` gain
  `starlight-website`, for work on the site, and `agent-docs-audit`, which
  checks the instruction set for drift. `.mcp.json` declares the `context7`
  and `astro-docs` documentation servers.

### Changed

#### CLI output

- **CLI** — **Breaking for scripts**: `proxy on` and `proxy off` no longer
  print a confirmation line. They emit only the shell statements to be
  evaluated, so `eval "$(proxy env on)"` is now silent. A script that looked
  for that confirmation needs updating.
- **CLI** — **Breaking for scripts**: the line `proxy use <key>` and
  `proxy switch` print after switching ends with the new `[ON]` / `[OFF]` tag,
  so its shape has changed.
- **CLI** — **Breaking for scripts**: `proxy ping` labels a non-2xx response
  `WARN (… ms)` in yellow; it used to be `OK` in green, like a success.
- **CLI**: Emoji are gone from the translation strings and from the headers of
  the reporting commands — `ping`, `diagnose`, `monitor`, `git`, `import`,
  `benchmark`, `speedtest` — and from the dashboard's port indicator. The
  prompt marker and the dashboard's title bar keep theirs. The status labels
  `proxy ping` prints are translated now, rather than fixed English.
- **CLI**: The rule printed between report blocks is a plain line of `-`
  instead of a bold cyan line of `=`, and report headers no longer start with
  spaces.

#### Documentation

- **Docs**: `docs/*.md` and `CONTRIBUTING.md` are plain CommonMark, readable on
  GitHub as before. The site's tabs, cards, steps and callouts are written as
  `<!--site:…-->` comments, which GitHub renders as nothing and the site
  generator expands.
- **Docs**: `docs/CONFIGURATION.md` lists the full default `ping_targets`, and
  its field table names every command that reads them.
- **Docs**: Corrected descriptions of what several commands do:
  - `benchmark` tests profiles one after another, probing each profile's ping
    targets at once — not every profile concurrently;
  - `Space` in `proxy dash` saves the selected profile as active without
    leaving; it does not update the shell, only `Enter` does;
  - `monitor`, on a failed health check or with no proxy variable set, saves
    the fastest reachable profile as active but does not change the shell it
    runs in;
  - `profile set` makes the profile active without re-applying the shell's
    variables;
  - `diagnose` lists the main proxy variables, not every one `proxy on` sets.
- **Docs**: The release notes in `.ai/GIT_WORKFLOW.md` describe the pipeline as
  it runs: the tag is created after the binaries build, so a failed build
  leaves no tag behind, and the recovery steps follow from that.

#### Dependencies

- **CLI**: `chacha20` 0.10.1, which was yanked from crates.io, is replaced by
  0.10.2.

#### Contributing

- **Contributing**: A change to `docs/`, the READMEs, `CONTRIBUTING.md` or a
  changelog also has to pass the site build
  (`cd website && npm ci && npm test && npm run build`), because those files
  are the site's content.
- **Contributing**: The changelog rule in `CONTRIBUTING.md` matches what CI
  enforces — changes to `shell/` and `configs/` need an entry too, in both
  `CHANGELOG.md` and `CHANGELOG.ru.md`. The pull request template uses the same
  `--locked` checks and `npm ci` as the rest of the documentation.
- **Contributing**: `CLAUDE.md` imports `AGENTS.md` instead of only linking to
  it, so Claude Code actually loads the project rules; the skills moved from
  `.agents/skills/` (now symlinks) to `.claude/skills/`, where Claude Code finds
  them. The `release-manager` skill updates the changelog link footer, and
  `cargo-audit` asks where a security entry belongs once a release is cut.
- **CI**: The branch policy compares a pull request against its merge base, so
  a documentation-only pull request is no longer blocked by other commits on
  the release branch.
- **CI**: Workflows default to read-only repository access; the `cargo audit`
  job gets only the extra permission it needs to report its results, which it
  previously failed to do.

### Removed

- **CLI**: The `env_on_msg` and `env_off_msg` translation keys, unused since
  `proxy on` and `proxy off` stopped printing a confirmation.
- **Contributing**: The `aur-packager` skill — the project publishes no AUR
  package — and the unused `.agents/mcp_config.json`.

## [2.2.2] — 2026-08-21

### Fixed
- **Release Automation**: Retry the Homebrew tap update with the fixed tag-name reference.

## [2.2.1] — 2026-08-21

### Fixed
- **Release Automation**: Fixed a release issue where the Homebrew formula was not successfully updated during the v2.2.0 release. Triggering a patch release to force the `mislav/bump-homebrew-formula-action` to run correctly and synchronize the tap.

## [2.2.0] — 2026-08-20

A correctness and hardening release. Several long-standing bugs are fixed,
including one that could destroy a user's saved profiles, and one shell
injection. Two changes affect scripts — see **Changed**.

### Fixed

- **A malformed `config.json` no longer destroys your profiles.** The first
  attempt at this fix was incomplete and was caught in review: `load()` stopped
  overwriting the file, but the next command that saved — `profile set/use/
  remove`, `import`, or Enter in the dashboard — still wrote the fallback
  defaults straight over it. A config that failed to parse is now marked as
  such, and any attempt to save over it fails with an explanation. Read-only
  commands keep working so the file can still be diagnosed. The same applies to
  `settings.json`, where overwriting also silently dropped `config_path` and
  pointed the tool at a different config.
- **`env on` exits non-zero when no profile is active.** It printed an error and
  exited `0`, so `proxy_on && deploy` carried on against an unproxied shell.
- **`profile set` no longer resets an existing profile's protocol.** `--protocol`
  had a default, so changing only the port silently rewrote an `http` profile to
  `socks5`.
- **IPv6 profiles produce usable URLs.** A bare `::1` was accepted but rendered
  as `http://::1:1080`, which no client can parse. Literals are now bracketed in
  URLs and left bare in the JVM `-Dhttp.proxyHost=` options, as each requires.
- **A proxy URL that cannot be applied is no longer treated as success.** The
  error from building the proxy was discarded, so the request went out
  *directly*: `monitor` reported a broken tunnel as healthy, `benchmark` ranked
  the broken profile fastest and `best` then selected it, and `status` and the
  dashboard showed the machine's real IP as though it were the proxy's.
- **The dashboard no longer corrupts its own display.** Pressing `s` ran a
  benchmark whose progress spinner writes to stderr on a timer, interleaving
  with ratatui's frames.
- **The dashboard selection no longer jumps when benchmark results arrive.** The
  list re-sorted without remapping the cursor, so Enter applied whichever
  profile had slid under it.
- **The shell wrapper reports the real exit status.** The re-apply step became
  the function's own status and inverted it: `proxy use work` returned failure
  on success with the proxy off, and `proxy use nope` returned success on
  failure with it on.
- **`run` no longer lets the child command retarget the manager.** The
  pre-parser scanned the whole command line, so `proxy run mytool --lang en`
  changed which config the manager itself loaded.
- **`speedtest` reports a dropped connection instead of a confident number.** A
  mid-stream transport error was indistinguishable from a clean end of body.
- **`diagnose` resolves hostnames.** The socket check only accepted IP literals,
  so a profile with a hostname always reported its port as closed. `load()`
  overwrote an unparsable config with the built-in defaults, so a single stray
  comma silently discarded every profile. A file that exists but fails to parse
  is now reported on stderr and left byte-for-byte intact. The same applies to
  `settings.json`.
- **`proxy --help` no longer prints the raw key `cmd_debug`.** The `debug`
  subcommand had no translation. A test now asserts every subcommand has a
  description in both languages.
- **`proxy run` accepts flags for the child command.** `proxy run curl -sS URL`
  failed with "unexpected argument '-s'"; arguments after the program name are
  now forwarded verbatim.
- **`-h` means `--help` everywhere again.** It was bound to `--host` on
  `profile set`, shadowing the help flag on that subcommand.
- **The dashboard no longer panics on an empty profile list.** Pressing an arrow
  key computed `len() - 1` on an empty list, which panicked inside the alternate
  screen and left the terminal unusable.
- **The terminal is restored however the dashboard exits.** An RAII guard plus a
  panic hook replace the previous success-path-only cleanup, which left users in
  raw mode on the alternate screen after any error or panic.
- **The dashboard exports the same variables as `env on`.** It set only
  `HTTP_PROXY`, `HTTPS_PROXY` and `ALL_PROXY`, silently dropping the lowercase
  names and the JVM options.
- **Importing an unrelated JSON file no longer invents profiles.** Because every
  field of the config has a serde default, any JSON object parsed successfully
  and "imported" the two built-in defaults.
- **`git on` / `git off` report failures.** The exit status of `git config` was
  discarded, so a missing `git` binary still printed a success message.
- **`monitor` no longer treats a 404 as healthy.** It now probes a configurable
  `health_check_url` that answers `204`, instead of special-casing one API's
  error response.
- Benchmark results distinguish an unreachable proxy from a merely slow one; the
  previous `9999` sentinel conflated the two.
- `NO_COLOR` is honoured per spec (any non-empty value) rather than only when set.

### Security

- **Shell injection in `env on` and the dashboard hand-off.** Output consumed by
  `eval "$(...)"` interpolated profile fields without quoting, so a host value
  from an edited or imported config could execute arbitrary commands. All values
  are now POSIX single-quoted, verified by a test that evaluates the generated
  script in a real shell.
- Profiles are validated before being persisted, from `profile set` and from
  imports alike: the host is checked against a character allowlist, the port must
  be non-zero, and the protocol must be one this tool supports.

### Added

- A documented branch model — `main` <- `release/X.Y.Z` <- task branches —
  described in `.ai/GIT_WORKFLOW.md` and enforced by
  `.github/workflows/branch-policy.yml`. Pull requests are checked for branch
  naming, a valid head/base pairing, and a changelog entry.
- Releases publish themselves. Merging a release branch into `main` makes CI
  verify the build, create the `vX.Y.Z` tag, build the four target binaries and
  bump the Homebrew tap. Tagging by hand still works and takes the same path.
- `--config-file`, `--settings-file` and `--lang` global options, with matching
  `TSPM_CONFIG`, `TSPM_SETTINGS` and `TSPM_LANG` environment variables. This
  makes the tool scriptable and lets tests run in full isolation.
- Configurable endpoints in `config.json`: `ipv4_api`, `ipv6_api`,
  `health_check_url` and `speedtest_url`. These were hard-coded.
- Full translations for `speedtest`, `monitor`, `git` and the TUI dashboard,
  which previously printed English regardless of the selected language.
- `CONTRIBUTING.md`, `SECURITY.md`, `AGENTS.md`, `CLAUDE.md`, issue and pull
  request templates, and Dependabot configuration.
- Tests grew from 9 to 126, including an end-to-end suite in `tests/cli.rs` that
  drives the real binary against an isolated config.

### Changed

Two changes may affect existing scripts:

- **`profile use` and `profile remove` exit non-zero on an unknown key.** They
  printed an error and exited `0`, so `proxy profile use "$p" || fallback` always
  took the success branch.
- **`profile set --host` no longer has the short form `-h`.** Use `--host`.

Also changed:

- `export` fails instead of printing nothing when no profile is active, so
  `proxy export envfile > .env` cannot silently truncate the file.
- `settings set --lang` rejects an unknown language instead of silently
  selecting Russian.
- The unused `enabled` field was removed from `config.json`. Existing configs
  keep loading; the field is ignored.
- `run` propagates the child's exit code exactly.
- Edition 2024, MSRV 1.88, and dependency upgrades: clap 4.6, tokio 1.48,
  reqwest 0.13, thiserror 2.0, dirs 6.0, colored 3.1, indicatif 0.18,
  dialoguer 0.12. The macOS config directory is unchanged by the `dirs` upgrade.
- Release builds use fat LTO, one codegen unit and stripped symbols.
- The release workflow tags only after every binary has built. Tagging first
  meant a failed build left the tag pushed with nothing published, and every
  retry then saw the tag and skipped the release permanently.
- CI gained an MSRV job, a `cargo audit` job, `--locked` builds and a docs build.
  Releases now verify the tag matches the crate version, ship `aarch64` Linux
  binaries, and publish SHA-256 checksums.

## [2.1.1] — 2026-08-18

### Added

- Bilingual CLI help messages.

### Changed

- Default configuration uses safe generic profiles.

[Unreleased]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v2.3.0...HEAD
[2.3.0]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v2.2.2...v2.3.0
[2.2.2]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v2.2.1...v2.2.2
[2.2.1]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v2.2.0...v2.2.1
[2.2.0]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v2.1.1...v2.2.0
[2.1.1]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/releases/tag/v2.1.1
