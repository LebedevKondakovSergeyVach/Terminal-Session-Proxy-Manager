# ⚡ Proxy CLI (Universal Rust Proxy Toolkit)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](README.md)

Универсальный, высокопроизводительный CLI-инструментарий на **Rust** для управления прокси-серверами, переключения профилей, диагностики сети и управления переменными окружения в **Zsh** и **Bash**.

---

## 📚 Разделы документации

Для удобства подробная документация разбита по специализированным руководствам:

- [📦 **Установка и Сборка** (`docs/INSTALLATION.md`)](docs/INSTALLATION.md) — Сборка из исходников, зависимости, интеграция в PATH.
- [🐚 **Интеграция с Шеллами** (`docs/SHELL_INTEGRATION.md`)](docs/SHELL_INTEGRATION.md) — Настройка `.zshrc`, `.bashrc`, Powerlevel10k instant prompt и prompt-сегменты.
- [⚙️ **Конфигурация** (`docs/CONFIGURATION.md`)](docs/CONFIGURATION.md) — Описание `settings.json`, схемы `config.json`, пресеты и поля.
- [📖 **Руководство по Использованию** (`docs/USAGE.md`)](docs/USAGE.md) — Подробный справочник всех команд (`status`, `profile`, `ping`, `diagnose`, `run`).

---

## ⚡ Быстрый старт (За 1 минуту)

### 1. Сборка и установка
```bash
cargo build --release
cp target/release/proxy-cli /opt/homebrew/bin/
```

### 2. Подключение к Zsh
Добавьте в `~/.zshrc`:
```zsh
eval "$(proxy-cli init zsh)"
```

### 3. Подключение к Bash
Добавьте в `~/.bashrc`:
```bash
eval "$(proxy-cli init bash)"
```

---

## 🚀 Основные команды

```bash
proxy status         # Показать статус, IPv4, IPv6 и Локацию
proxy status --json  # Статус в формате JSON
proxy on             # Включить прокси для текущей вкладки
proxy off            # Выключить прокси
proxy use v2ray      # Переключить профиль на v2rayN
proxy use throne     # Переключить профиль на Throne
proxy ping           # Параллельный замер задержки до сервисов
proxy diagnose       # Расширенная диагностика TCP сокета и API
proxy run -- <cmd>   # Выполнить команду через прокси
```

---

## 📂 Структура репозитория

```text
proxy-cli-rs/
├── .gitignore              # Исключения для Git
├── Cargo.toml              # Конфигурация Rust и зависимости
├── settings.json           # Файл с указанием пути к активному конфигу
├── README.md               # Главный файл документации
├── docs/                   # Подробные руководства по категориям
│   ├── INSTALLATION.md
│   ├── SHELL_INTEGRATION.md
│   ├── CONFIGURATION.md
│   └── USAGE.md
├── configs/                # Пресеты и шаблоны конфигураций
│   ├── config.default.json
│   └── config.throne-v2ray.json
├── shell/                  # Скрипты инициализации
│   ├── proxy-cli.zsh
│   └── proxy-cli.bash
└── src/                    # Исходный код Rust engine
    ├── main.rs
    ├── config.rs
    ├── status.rs
    ├── ping.rs
    ├── diagnose.rs
    ├── env_cmd.rs
    └── init.rs
```

---

## 📄 Лицензия

Проект распространяется под лицензией MIT.
