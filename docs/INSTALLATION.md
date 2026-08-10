# 📦 Установка и Сборка

## Требования
- Rust 1.70+ (`cargo`, `rustc`)
- Zsh или Bash

---

## Сборка и установка

```bash
# 1. Клонирование
git clone https://github.com/LebedevSergeyVach/Proxy-CLI-rs.git
cd Proxy-CLI-rs

# 2. Релизная компиляция
cargo build --release

# 3. Установка бинарника (один из вариантов)
cp target/release/proxy-cli /opt/homebrew/bin/   # macOS Homebrew
# или:
cp target/release/proxy-cli ~/.cargo/bin/        # Cargo PATH
```

## Проверка
```bash
proxy-cli --version
# proxy-cli 0.2.0
```
