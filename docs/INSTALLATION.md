# 📦 Installation (macOS & Linux)

## 🍏 macOS / Linux (Homebrew)

The easiest way to install is via Homebrew using our custom tap:

```bash
brew install LebedevKondakovSergeyVach/tap/terminal-session-proxy-manager
```

## 🦀 Via Cargo (Any OS)

If you have Rust installed, you can install the binary directly from GitHub:

```bash
cargo install --git https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git terminal-session-proxy-manager
```

---

## 🛠️ Building from Source

If you prefer to compile the application locally:

```bash
# 1. Clone repository
git clone https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager

# 2. Compile release binary
cargo build --release
```bash
cargo install --path .
```

---

## 🐚 Shell Integration (Required)

No matter how you installed the application, you must add the initialization script to your shell configuration file to enable the interactive `proxy` command.

**For Zsh** (Add to `~/.zshrc`):
```bash
eval "$(terminal-session-proxy-manager init zsh)"
```

**For Bash** (Add to `~/.bashrc`):
```bash
eval "$(terminal-session-proxy-manager init bash)"
```

*Don't forget to restart your terminal or run `source ~/.zshrc` to apply the changes!*

## 🔍 Verification
```bash
terminal-session-proxy-manager --version
# terminal-session-proxy-manager 1.1.1

proxy status
```
