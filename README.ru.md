<p align="center">
  🇬🇧 <a href="README.md">English</a> | 🇷🇺 <b>Русский</b>
</p>

# ⚡ Terminal Session Proxy Manager

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](README.md)

Универсальный менеджер прокси-сессий для терминала на **Rust** для переключения прокси, контроля IP/локации, тестирования скорости и управления переменными окружения в **macOS** и **Linux** (Zsh / Bash).

## ✨ Особенности (Features)

- **Высокая скорость**: Написана на Rust, минимальное потребление ресурсов и мгновенный отклик.
- **Интерактивность**: Удобное TTY-меню для выбора профилей.
- **Авто-восстановление**: Мониторинг здоровья прокси в реальном времени с автоматическим переключением на самый быстрый узел (auto-heal).
- **Диагностика**: Встроенные утилиты для проверки пинга и реальной скорости скачивания.
- **Универсальный экспорт**: Команды для быстрой настройки Docker, cURL, Git и `.env` файлов.
- **Двуязычность**: Полная поддержка английского и русского языков.

---

## ⚡ Быстрый старт

### 📦 Установка

**macOS / Linux (через Homebrew):**
```bash
brew install LebedevSergeyVach/tap/terminal-session-proxy-manager
```

**Arch Linux (через AUR):**
```bash
yay -S terminal-session-proxy-manager
```

**Через Cargo:**
```bash
cargo install --git https://github.com/LebedevSergeyVach/Terminal-Session-Proxy-Manager.git terminal-session-proxy-manager
```

Или собрать локально из исходников:
```bash
git clone https://github.com/LebedevSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager
cargo install --path .
```

### 🐚 Интеграция с оболочкой

Чтобы команды `proxy` могли напрямую управлять вашей текущей сессией терминала, добавьте скрипт инициализации в конфигурационный файл вашей оболочки:

**Для Zsh** (`~/.zshrc`):
```bash
eval "$(terminal-session-proxy-manager init zsh)"
```

**Для Bash** (`~/.bashrc`):
```bash
eval "$(terminal-session-proxy-manager init bash)"
```

*После добавления обязательно перезапустите терминал или выполните `source ~/.zshrc`.*

---

## 🚀 Команды

| Команда | Описание |
| :--- | :--- |
| `proxy status` | Проверить статус прокси, IPv4, IPv6 и локацию |
| `proxy status --json` | Вывести статус в формате JSON |
| `proxy on` | Включить прокси для текущей сессии |
| `proxy off` | Выключить прокси |
| `proxy git <on/off/status>` | Управление глобальными настройками прокси в Git |
| `proxy export <docker/curl/env>` | Экспорт конфигураций в форматы Docker, cURL, `.env` |
| `proxy speedtest` | Замер реальной пропускной способности (Мб/с) |
| `proxy monitor` | Проверка здоровья и авто-переключение при сбое (auto-heal) |
| `proxy lang <ru/en>` | Переключить язык интерфейса (`ru`, `en`) |
| `proxy use <profile>` | Переключить профиль (`throne`, `v2ray`) |
| `proxy dash` | Запустить интерактивный TUI Дашборд (монитор, пинг, выбор) |
| `proxy switch` | Интерактивное меню выбора профиля стрелочками |
| `proxy benchmark` | Измерить пинг и доступность всех профилей |
| `proxy best` | Автоматически выбрать самый быстрый прокси |
| `proxy import <source>` | Импортировать профили из JSON файла или URL |
| `proxy ping` | Параллельный пинг целевых сервисов |
| `proxy diagnose` | Проверить локальный сокет и доступность API |
| `proxy run -- <cmd>` | Выполнить команду через прокси |
| `terminal-session-proxy-manager completions zsh` | Сгенерировать автодополнение |

---

## 📚 Документация

- [📦 **Установка (macOS & Linux)**](docs/INSTALLATION.ru.md) — Подробная сборка, настройки PATH.
- [🐚 **Интеграция с оболочками**](docs/SHELL_INTEGRATION.ru.md) — Zsh, Bash, Powerlevel10k.
- [⚙️ **Конфигурация**](docs/CONFIGURATION.ru.md) — Схема `settings.json` и `config.json`.
- [📖 **Справочник команд**](docs/USAGE.ru.md) — Описание всех подкоманд.

---

## 📄 Лицензия

[GNU AGPLv3](LICENSE)
