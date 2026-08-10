# ⚡ Proxy CLI

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](README.md)

Утилита на **Rust** для переключения прокси, контроля IP/локации, диагностики и управления переменными окружения в Zsh и Bash.

---

## ⚡ Быстрый старт

```bash
# 1. Сборка и установка
cargo build --release
cp target/release/proxy-cli /opt/homebrew/bin/

# 2. Подключение к Zsh (~/.zshrc)
eval "$(proxy-cli init zsh)"

# 3. Подключение к Bash (~/.bashrc)
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
| `proxy switch` | Интерактивный меню выбора профиля |
| `proxy benchmark` | Измерить пинг и доступность всех профилей |
| `proxy best` | Автоматически выбрать самый быстрый прокси |
| `proxy ping` | Параллельный пинг целевых сервисов |
| `proxy diagnose` | Проверить локальный сокет и доступность API |
| `proxy run -- <cmd>` | Выполнить команду через прокси |
| `proxy-cli completions zsh` | Сгенерировать автодополнение для zsh |

---

## 📚 Документация

- [📦 **Установка**](docs/INSTALLATION.md) — Сборка, зависимости, пути PATH.
- [🐚 **Интеграция с оболочками**](docs/SHELL_INTEGRATION.md) — Zsh, Bash, Powerlevel10k.
- [⚙️ **Конфигурация**](docs/CONFIGURATION.md) — Схема `settings.json` и `config.json`.
- [📖 **Справочник команд**](docs/USAGE.md) — Полное описание всех подкоманд.

---

## 📄 Лицензия

[GNU AGPLv3](LICENSE)
