---
name: release-manager
description: >-
  Automates the workflow of creating a new release branch, bumping versions, and updating changelogs.
---

# release-manager

When the user asks you to cut a new release, strictly follow this procedure:

1.  **Check current branch**: Ensure you are on `main`. If not, checkout `main` and pull latest changes.
2.  **Determine version**: Ask the user for the new version number (e.g., `x.y.z`) if not provided.
3.  **Create release branch**: Create a new branch named `release/<new_version>`.
4.  **Bump versions**: 
    - Update `version = "x.y.z"` in `Cargo.toml`.
    - Run `cargo check` to automatically update `Cargo.lock`.
5.  **Update Changelog**:
    - Rename the `## [Unreleased]` section header in `CHANGELOG.md` and `CHANGELOG.ru.md` to `## [x.y.z] — YYYY-MM-DD`. Note the em dash: every existing entry uses one, and a hyphen would break the file's consistency.
    - Create a new empty `## [Unreleased]` section above it in both files.
    - **Update the link-reference footer at the bottom of both files.** Repoint `[Unreleased]` at the new tag and add a line for the release itself:

      ```
      [Unreleased]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/vx.y.z...HEAD
      [x.y.z]: https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/compare/v<previous>...vx.y.z
      ```

      Skipping this is not cosmetic: a heading whose reference is missing renders as literal `[x.y.z]` text with no link, on GitHub and on the documentation site alike. Both changelogs are currently missing references for 2.2.1 and 2.2.2 for exactly this reason.
6.  **Verify**: Invoke the `verify-project` skill (run `cargo fmt`, `clippy`, and `test`) to ensure the release is stable.
7.  **Commit**: Commit the changes with the message `chore: bump version to x.y.z`.
8.  **Push & PR**: Push the branch to origin and instruct the user to open a Pull Request (or use the GitHub MCP server to open it automatically).
