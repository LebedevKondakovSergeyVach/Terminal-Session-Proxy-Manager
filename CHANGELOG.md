# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.3.0] — 2026-08-20

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
[2.3.0]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v2.1.1...v2.3.0
[2.1.1]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/releases/tag/v2.1.1
