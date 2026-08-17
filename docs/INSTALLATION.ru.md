# 📦 Установка (macOS & Linux)

## 🍏 macOS / Linux (Homebrew)

Самый простой способ установки через наш собственный репозиторий (tap):

```bash
brew install LebedevSergeyVach/tap/terminal-session-proxy-manager
```

## 🐧 Arch Linux (AUR)

Если вы используете Arch Linux, пакет можно установить напрямую из AUR через ваш любимый помощник (например, `yay` или `paru`):

```bash
yay -S terminal-session-proxy-manager
```

## 🦀 Через Cargo (Любая ОС)

Если у вас установлен Rust, вы можете установить бинарный файл напрямую с GitHub:

```bash
cargo install --git https://github.com/LebedevSergeyVach/Terminal-Session-Proxy-Manager.git terminal-session-proxy-manager
```

---

## 🛠️ Сборка из исходников

Если вы предпочитаете скомпилировать программу локально:

```bash
# 1. Клонирование
git clone https://github.com/LebedevSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager

# 2. Релизная компиляция
cargo build --release
```bash
cargo install --path .
```

---

## 🐚 Интеграция с оболочкой (Обязательно)

Независимо от того, как вы установили программу, вам необходимо добавить скрипт инициализации в конфигурационный файл вашей оболочки, чтобы работала интерактивная команда `proxy`.

**Для Zsh** (Добавьте в `~/.zshrc`):
```bash
eval "$(terminal-session-proxy-manager init zsh)"
```

**Для Bash** (Добавьте в `~/.bashrc`):
```bash
eval "$(terminal-session-proxy-manager init bash)"
```

*Не забудьте перезапустить терминал или выполнить `source ~/.zshrc` для применения изменений!*

## 🔍 Проверка работы
```bash
terminal-session-proxy-manager --version
# terminal-session-proxy-manager 1.1.1

proxy status
```
