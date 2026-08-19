# 📦 Installation (macOS & Linux)

## 🍏 Homebrew

The easiest route, on both macOS and Linux:

```bash
brew install LebedevKondakovSergeyVach/tap/terminal-session-proxy-manager
```

To upgrade later:

```bash
brew upgrade terminal-session-proxy-manager
```

## 🦀 Cargo

Requires Rust **1.85 or newer** (the project uses edition 2024). Install a
toolchain from [rustup.rs](https://rustup.rs), then:

```bash
cargo install --git https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
```

This puts the binary in `~/.cargo/bin`, which rustup normally adds to your
`PATH`. If `terminal-session-proxy-manager` is not found afterwards, see
[PATH setup](#-path-setup) below.

## 📥 Prebuilt binaries

Download an archive for your platform from the
[releases page](https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/releases).
Builds are published for macOS (x86_64, arm64) and Linux (x86_64, arm64).

Each archive ships with a `.sha256` file. Verify before installing:

```bash
shasum -a 256 -c terminal-session-proxy-manager-macos-arm64.tar.gz.sha256

tar -xzf terminal-session-proxy-manager-macos-arm64.tar.gz
sudo mv terminal-session-proxy-manager /usr/local/bin/
```

On macOS, Gatekeeper may quarantine a downloaded binary. Clear it with:

```bash
xattr -d com.apple.quarantine /usr/local/bin/terminal-session-proxy-manager
```

---

## 🛠️ Building from source

```bash
git clone https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager

# Install into ~/.cargo/bin
cargo install --path .

# Or just build, leaving the binary in target/release/
cargo build --release
```

---

## 🔧 PATH setup

If the binary is installed but not found, add its directory to your `PATH`.

For a Cargo install (`~/.zshrc` or `~/.bashrc`):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

For a manual install into `/usr/local/bin`, that directory is usually on `PATH`
already. Confirm which binary is being used with:

```bash
which terminal-session-proxy-manager
```

---

## 🐚 Shell integration (required)

However you installed it, add the init script to your shell configuration.
Without it the `proxy` command does not exist and nothing can change your
session's environment — no process can modify its parent shell.

**Zsh** (`~/.zshrc`):

```bash
eval "$(terminal-session-proxy-manager init zsh)"
```

**Bash** (`~/.bashrc`):

```bash
eval "$(terminal-session-proxy-manager init bash)"
```

Restart your terminal or run `source ~/.zshrc` to apply. Details and the full
list of functions are in [SHELL_INTEGRATION.md](SHELL_INTEGRATION.md).

---

## 🔍 Verifying the installation

```bash
terminal-session-proxy-manager --version
type proxy          # should report a shell function
proxy config path   # shows which config.json is in use
proxy status
```

---

## 🗑️ Uninstalling

```bash
# Homebrew
brew uninstall terminal-session-proxy-manager

# Cargo
cargo uninstall terminal-session-proxy-manager

# Manual
sudo rm /usr/local/bin/terminal-session-proxy-manager
```

Then remove the `eval "$(... init ...)"` line from your shell rc file.

Configuration is left behind on purpose. Remove it too if you want:

```bash
# macOS
rm -rf ~/Library/Application\ Support/terminal-session-proxy-manager
# Linux
rm -rf ~/.config/terminal-session-proxy-manager
```
