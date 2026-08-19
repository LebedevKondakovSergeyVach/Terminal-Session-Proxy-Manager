# Workflow, Linting & CI/CD Guide for AI Agents

**ATTENTION AI AGENTS:**
Strictly adhere to the following workflow rules when modifying this repository. Failure to comply will result in broken CI pipelines and disrupted user experiences.

## 1. Local Testing & Verification
Any change to business logic, JSON parsing, or configuration structures is prone to breaking unit tests. 
Before finalizing your task or pushing commits, **YOU MUST** verify the codebase health:
```bash
# 1. Run all unit and integration tests
cargo test

# 2. Check for compilation warnings or dead code
cargo check
```
If tests fail or warnings appear (e.g., unused fields, unused imports), you must fix them immediately.

## 2. Linting and Formatting
This project adheres to standard Rust formatting and strict linting rules. 
Before pushing code, ensure you run the linter and formatter:
```bash
# Apply standard formatting
cargo fmt

# Run the official Rust linter and fix basic issues automatically
cargo clippy --fix --allow-dirty --allow-staged
# Or just run it to view warnings:
cargo clippy -- -D warnings
```
Do not ignore Clippy warnings. Refactor code to satisfy idiomatic Rust standards.

## 3. Localization (i18n) Rules
This CLI supports dual languages natively. **Never hardcode raw strings** into `println!` or UI rendering logic.
- **CLI Commands & Arguments:** In `src/cli.rs`, `clap` doc comments (`///`) must strictly follow the `English | Русский` format.
  - *Example:* `/// Manage active config.json file | Управление файлом config.json`
- **Application Output:** Use the `I18n::t("key")` function for all terminal outputs.
- **JSON Dictionaries:** If you add a new string key, you **must** add it to BOTH `locales/en.json` and `locales/ru.json` simultaneously to keep dictionaries symmetrical.

## 4. UI/Graphics & Documentation (README)
The `README.md` (English) and `README.ru.md` (Russian) are the main entry points for users.
- If you add a new CLI command or feature, update both READMEs.
- **Terminal Screenshots:** The project relies on visual demonstrations (e.g., the TUI Dashboard). If you drastically alter the TUI (`src/cmd/dash.rs`), notify the user that a new screenshot is required. 
- *Caching Tip:* To bypass aggressive GitHub image caching (`Camo`), rename the screenshot file entirely (e.g., `assets/proxy_dashboard_v3.png`) and update the markdown links rather than overriding the existing file.

## 5. Automated CI/CD & Releases
The project uses GitHub Actions (`.github/workflows/release.yml`) to automatically compile cross-platform binaries and deploy them to a Homebrew tap.
**The Release Process:**
To trigger a new release, you must bump the version and push a specific git tag.
1. Update `version = "X.Y.Z"` in `Cargo.toml`.
2. Run `cargo check` to automatically sync `Cargo.lock`.
3. Commit the bump: `git commit -am "chore: bump version to X.Y.Z"`.
4. Create an annotated or lightweight tag: `git tag vX.Y.Z`.
5. Push the tag to trigger the pipeline: `git push origin main --tags`.

*Note: Homebrew updates are fully automated by the GitHub Action. You do not need to manually touch the `homebrew-tap` repository after tagging a release.*
