# Contributing

Thanks for taking the time. This is a small project, so the process is short.

## Getting set up

Requires Rust 1.88 or newer (edition 2024). Install via [rustup](https://rustup.rs).

```bash
git clone https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager
cargo build
cargo test
```

To try your build without installing it:

```bash
cargo run -- status
cargo run -- profile list
```

## Before you open a pull request

Run what CI runs. Warnings are build failures.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

## Things that are easy to get wrong

**Every user-facing string is translated.** Use `i18n.t("key")`, or
`i18n.format("key", &[a, b])` when it has `{}` placeholders, and add the key to
**both** `locales/en.json` and `locales/ru.json`. Tests assert the two files
define identical key sets, so a one-sided addition fails the build. Adding a
subcommand also needs a `cmd_<name>` key in both files, otherwise `--help`
prints the raw key.

**Never overwrite a config that failed to parse.** Defaults are written only
when a file is absent. Overwriting an unparsable `config.json` discards every
profile the user has.

**Quote anything a shell will evaluate.** The `env on` output is consumed by the
shell's `eval`, and profile fields come from a file the user can edit or import
from a URL. Route values through `proxy_env::shell_quote`.

**Failure means a non-zero exit code.** Return `Err` rather than printing an
error and returning `Ok` — people write `proxy profile use "$p" || fallback`.

**No private data in defaults.** `AppConfig::default` may reference only
loopback addresses and well-known public endpoints.

## Tests

Name tests as sentences describing the guarantee — `using_an_unknown_profile_exits_non_zero`
rather than `test_profile` — and assert one thing per test.

- Pure logic goes in a `#[cfg(test)] mod tests` next to the code.
- Anything involving argv, exit codes, or files goes in `tests/cli.rs`.
- Integration tests must isolate state with `TSPM_CONFIG` and `TSPM_SETTINGS`;
  use the `Cli` helper at the top of `tests/cli.rs`. A test must never touch the
  real `~/.config`.
- Please don't add tests that need network access. They are slow and flaky.

When fixing a bug, add the test that fails without the fix.

## Documentation

If you change commands or configuration, update `README.md` **and**
`README.ru.md`, the matching file in `docs/` and its `.ru.md` twin, and add a
`CHANGELOG.md` entry under `Unreleased`.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`,
`docs:`, `chore:`, `refactor:`, `test:`, `perf:`, `ci:`.

## Releases

Maintainers only, and never as a side effect of another change — pushing a tag
publishes binaries. The process is in
[`.ai/WORKFLOW_GUIDE.md`](.ai/WORKFLOW_GUIDE.md).

## Using an AI agent

Point it at [`AGENTS.md`](AGENTS.md) first; it encodes the rules above in the
form agents follow.
