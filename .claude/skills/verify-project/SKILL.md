---
name: verify-project
description: Use before claiming any change is done, fixed, passing or ready to commit - Rust code, locales, shell or config files, dependencies, documentation, changelogs or the website.
---

# verify-project

## 1. The crate — always

```bash
cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked
```

CI adds `--all-features` to clippy and tests and supplies `-D warnings` through
`RUSTFLAGS`. `Cargo.toml` declares no `[features]` table, so the two agree.
It also runs an MSRV job on `rust-version`; if you used a newly stabilised std
API, say that the MSRV job is the one to watch.

## 2. The website — when the change touched its sources

Required when the change touched any of:
`website/`, `docs/`, `README.md`, `README.ru.md`, `CONTRIBUTING.md`,
`CHANGELOG.md`, `CHANGELOG.ru.md`, `assets/`

```bash
cd website && npm ci && npm test && npm run build
```

`npm run build` regenerates every page and fails on a dead internal link. The
same commands run in `.github/workflows/website.yml`, which also runs
`npm audit --audit-level=high`. A release always qualifies.

After the build, `git status --short website/src/content/docs` must show nothing
new. CI's diff check detects regeneration changes to tracked files, but an
unchanged tracked generated page can pass it. Check manifest destinations too:

```bash
node --input-type=module <<'JS'
import { execFileSync } from 'node:child_process';
import { PAGES } from './website/scripts/docs-manifest.mjs';
const tracked = new Set(execFileSync('git', ['ls-files', '-z'], { encoding: 'utf8' }).split('\0'));
const generated = PAGES.map(({ dest }) => `website/src/content/docs/${dest}`);
const bad = generated.filter((path) => tracked.has(path));
if (bad.length) {
  console.error('Generated pages are tracked:', bad.join(', '));
  process.exitCode = 1;
} else {
  console.log('No manifest-generated page is tracked.');
}
JS
```

Run that snippet from the repository root.

## 3. What the gate does not cover

Check these by hand when they apply:

- `shell/*.zsh`, `shell/*.bash` changed → `zsh -n` / `bash -n` on them (tests
  check only the `init` output).
- `src/`, `locales/`, `shell/`, `configs/` or `Cargo.toml` changed → entries in
  **both** changelogs. Direct commits to the release branch are not checked by CI.
- Agent instruction files changed → the `agent-docs-audit` skill.
- Release workflows or `.github/scripts/` changed →
  `python3 -m unittest discover -s .github/scripts -p 'test_*.py'`.

## 4. Reporting

Report each gate and its exit status; include relevant output for failures.
Name checks you did not run and why. If something fails, report the error —
do not describe the task as finished.
