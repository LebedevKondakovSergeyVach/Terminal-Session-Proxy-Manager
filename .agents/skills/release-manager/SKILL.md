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
    - Rename the `## [Unreleased]` section header in `CHANGELOG.md` to `## [x.y.z] - YYYY-MM-DD`.
    - Create a new empty `## [Unreleased]` section above it.
6.  **Verify**: Invoke the `verify-project` skill (run `cargo fmt`, `clippy`, and `test`) to ensure the release is stable.
7.  **Commit**: Commit the changes with the message `chore: bump version to x.y.z`.
8.  **Push & PR**: Push the branch to origin and instruct the user to open a Pull Request (or use the GitHub MCP server to open it automatically).
