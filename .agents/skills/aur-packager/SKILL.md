---
name: aur-packager
description: >-
  Provides guidelines and workflows for updating the Arch User Repository (AUR) PKGBUILD for this project.
---

# aur-packager

When the user asks to publish or update the AUR package, follow these steps:

1.  **Retrieve Release Info**: Get the latest tagged version and its source tarball URL from GitHub Releases.
2.  **Calculate Checksums**: Download the tarball (`.tar.gz`) and generate the `sha256sums` (e.g., using `curl -sL <url> | sha256sum`).
3.  **Update PKGBUILD**:
    - Modify the `pkgver` variable to match the new version.
    - Reset `pkgrel=1`.
    - Update `sha256sums` with the newly calculated hash.
4.  **Local Testing**:
    - If running on Arch Linux or using a Docker Arch container, run `makepkg -si` to ensure the package builds correctly.
5.  **Generate .SRCINFO**: Run `makepkg --printsrcinfo > .SRCINFO` to update the metadata.
6.  **Commit and Push**: Commit the `PKGBUILD` and `.SRCINFO` to the local AUR git repository, then run `git push` to publish to the Arch User Repository.
