# 📦 Installation & Building (macOS & Linux)

## 📋 Requirements
- **Rust 1.70+** (`cargo`, `rustc`)
- **Shell**: `zsh` or `bash`

---

## 🛠️ Building from Source

```bash
# 1. Clone repository
git clone https://github.com/LebedevSergeyVach/Proxy-CLI-rs.git
cd Proxy-CLI-rs

# 2. Compile release binary
cargo build --release
```

---

## 🍏 Installation on macOS

```bash
# Option A: Homebrew bin (Recommended)
cp target/release/proxy-cli /opt/homebrew/bin/

# Option B: Cargo bin
cp target/release/proxy-cli ~/.cargo/bin/
```

### Add to `~/.zshrc`:
```zsh
eval "$(proxy-cli init zsh)"
```

---

## 🐧 Installation on Linux (Ubuntu/Debian/Fedora/Arch)

```bash
# Option A: System PATH (For all users)
sudo cp target/release/proxy-cli /usr/local/bin/

# Option B: User Cargo bin
cp target/release/proxy-cli ~/.cargo/bin/
```

### Add to `~/.bashrc` or `~/.zshrc`:
```bash
eval "$(proxy-cli init bash)"
```

---

## 🔍 Verification
```bash
proxy-cli --version
# proxy-cli 1.1.1

proxy status
```
