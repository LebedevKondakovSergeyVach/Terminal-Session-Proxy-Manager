# 📦 Installation & Building (macOS & Linux)

## 📋 Requirements
- **Rust 1.70+** (`cargo`, `rustc`)
- **Shell**: `zsh` or `bash`

---

## 🛠️ Building from Source

```bash
# 1. Clone repository
git clone https://github.com/LebedevSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager

# 2. Compile release binary
cargo build --release
```

---

## 🍏 Installation on macOS

```bash
# Option A: Homebrew bin (Recommended)
cp target/release/terminal-session-proxy-manager /opt/homebrew/bin/

# Option B: Cargo bin
cp target/release/terminal-session-proxy-manager ~/.cargo/bin/
```

### Add to `~/.zshrc`:
```zsh
eval "$(terminal-session-proxy-manager init zsh)"
```

---

## 🐧 Installation on Linux (Ubuntu/Debian/Fedora/Arch)

```bash
# Option A: System PATH (For all users)
sudo cp target/release/terminal-session-proxy-manager /usr/local/bin/

# Option B: User Cargo bin
cp target/release/terminal-session-proxy-manager ~/.cargo/bin/
```

### Add to `~/.bashrc` or `~/.zshrc`:
```bash
eval "$(terminal-session-proxy-manager init bash)"
```

---

## 🔍 Verification
```bash
terminal-session-proxy-manager --version
# terminal-session-proxy-manager 1.1.1

proxy status
```
