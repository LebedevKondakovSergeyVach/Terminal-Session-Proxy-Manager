## What changed

<!-- One or two sentences. Link the issue this closes, if any. -->

## Why

<!-- The problem this solves. -->

## Checklist

- [ ] `cargo fmt --all` leaves no diff
- [ ] `cargo clippy --all-targets -- -D warnings` is clean
- [ ] `cargo test` passes
- [ ] New user-facing strings go through `I18n::t` / `I18n::format` and exist in
      **both** `locales/en.json` and `locales/ru.json`
- [ ] New or changed CLI commands are reflected in `README.md`, `README.ru.md`
      and `docs/`
- [ ] `CHANGELOG.md` has an entry under `Unreleased`
