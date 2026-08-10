# 📦 Установка и Сборка (macOS & Linux)

## 📋 Требования
- **Rust 1.70+** (`cargo`, `rustc`)
- **Оболочка**: `zsh` или `bash`

---

## 🛠️ Сборка из исходников

```bash
# 1. Клонирование
git clone https://github.com/LebedevSergeyVach/Proxy-CLI-rs.git
cd Proxy-CLI-rs

# 2. Релизная компиляция
cargo build --release
```

---

## 🍏 Установка на macOS

```bash
# Вариант A: Homebrew bin (Рекомендуется)
cp target/release/proxy-cli /opt/homebrew/bin/

# Вариант B: Cargo bin
cp target/release/proxy-cli ~/.cargo/bin/
```

### Добавление в `~/.zshrc`:
```zsh
eval "$(proxy-cli init zsh)"
```

---

## 🐧 Установка на Linux (Ubuntu/Debian/Fedora/Arch)

```bash
# Вариант A: Системный PATH (Для всех пользователей)
sudo cp target/release/proxy-cli /usr/local/bin/

# Вариант B: Пользовательский Cargo bin
cp target/release/proxy-cli ~/.cargo/bin/
```

### Добавление в `~/.bashrc` или `~/.zshrc`:
```bash
eval "$(proxy-cli init bash)"
```

---

## 🔍 Проверка работы
```bash
proxy-cli --version
# proxy-cli 0.2.0

proxy status
```
