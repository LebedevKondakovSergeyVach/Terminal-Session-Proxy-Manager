# 📦 Установка и Сборка (macOS & Linux)

## 📋 Требования
- **Rust 1.70+** (`cargo`, `rustc`)
- **Оболочка**: `zsh` или `bash`

---

## 🛠️ Сборка из исходников

```bash
# 1. Клонирование
git clone https://github.com/LebedevSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager

# 2. Релизная компиляция
cargo build --release
```

---

## 🍏 Установка на macOS

```bash
# Вариант A: Homebrew bin (Рекомендуется)
cp target/release/terminal-session-proxy-manager /opt/homebrew/bin/

# Вариант B: Cargo bin
cp target/release/terminal-session-proxy-manager ~/.cargo/bin/
```

### Добавление в `~/.zshrc`:
```zsh
eval "$(terminal-session-proxy-manager init zsh)"
```

---

## 🐧 Установка на Linux (Ubuntu/Debian/Fedora/Arch)

```bash
# Вариант A: Системный PATH (Для всех пользователей)
sudo cp target/release/terminal-session-proxy-manager /usr/local/bin/

# Вариант B: Пользовательский Cargo bin
cp target/release/terminal-session-proxy-manager ~/.cargo/bin/
```

### Добавление в `~/.bashrc` или `~/.zshrc`:
```bash
eval "$(terminal-session-proxy-manager init bash)"
```

---

## 🔍 Проверка работы
```bash
terminal-session-proxy-manager --version
# terminal-session-proxy-manager 1.1.1

proxy status
```
