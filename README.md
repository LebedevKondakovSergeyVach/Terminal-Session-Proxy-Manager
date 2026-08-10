# ⚡ Proxy CLI

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](README.md)

Универсальная CLI-утилита на **Rust** для переключения прокси, контроля IP/локации, тестирования скорости и управления переменными окружения в **macOS** и **Linux** (Zsh / Bash).

---

## ⚡ Быстрый старт

### 🍏 macOS
```bash
cargo build --release
cp target/release/proxy-cli /opt/homebrew/bin/

# Добавить в ~/.zshrc:
eval "$(proxy-cli init zsh)"
```

### 🐧 Linux
```bash
cargo build --release
sudo cp target/release/proxy-cli /usr/local/bin/

# Добавить в ~/.bashrc (или ~/.zshrc):
eval "$(proxy-cli init bash)"
```

---

## 🚀 Команды

| Команда | Описание |
| :--- | :--- |
| `proxy status` | Проверить статус прокси, IPv4, IPv6 и локацию |
| `proxy status --json` | Вывести статус в формате JSON |
| `proxy on` | Включить прокси для текущей сессии |
| `proxy off` | Выключить прокси |
| `proxy use <profile>` | Переключить профиль (`throne`, `v2ray`) |
| `proxy switch` | Интерактивное меню выбора профиля |
| `proxy benchmark` | Измерить пинг и доступность всех профилей |
| `proxy best` | Автоматически выбрать самый быстрый прокси |
| `proxy import <file/url>` | Импортировать профили из JSON файла или URL |
| `proxy ping` | Параллельный пинг целевых сервисов |
| `proxy diagnose` | Проверить локальный сокет и доступность API |
| `proxy run -- <cmd>` | Выполнить команду через прокси |
| `proxy-cli completions zsh` | Сгенерировать автодополнение |

---

## 📚 Документация

- [📦 **Установка (macOS & Linux)**](docs/INSTALLATION.md) — Подробная сборка, настройки PATH.
- [🐚 **Интеграция с оболочками**](docs/SHELL_INTEGRATION.md) — Zsh, Bash, Powerlevel10k.
- [⚙️ **Конфигурация**](docs/CONFIGURATION.md) — Схема `settings.json` и `config.json`.
- [📖 **Справочник команд**](docs/USAGE.md) — Описание всех подкоманд.

---

## 📄 Лицензия

[GNU AGPLv3](LICENSE)
