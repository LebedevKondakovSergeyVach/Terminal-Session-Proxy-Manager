# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.2.0] — 2026-08-19

A correctness and hardening release. Several long-standing bugs are fixed,
including one that could destroy a user's saved profiles.

### Fixed

- **A malformed `config.json` no longer destroys your profiles.** `load()`
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
- CI gained an MSRV job, a `cargo audit` job, `--locked` builds and a docs build.
  Releases now verify the tag matches the crate version, ship `aarch64` Linux
  binaries, and publish SHA-256 checksums.

## [2.1.1] — 2026-08-18

### Added

- Bilingual CLI help messages.

### Changed

- Default configuration uses safe generic profiles.

[Unreleased]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v2.2.0...HEAD
[2.2.0]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v2.1.1...v2.2.0
[2.1.1]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/releases/tag/v2.1.1
