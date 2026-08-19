# 📦 Установка (macOS и Linux)

## 🍏 Homebrew

Самый простой способ, работает и на macOS, и на Linux:

```bash
brew install LebedevKondakovSergeyVach/tap/terminal-session-proxy-manager
```

Обновление:

```bash
brew upgrade terminal-session-proxy-manager
```

## 🦀 Cargo

Требуется Rust **1.85 или новее** (проект использует edition 2024). Установите
toolchain через [rustup.rs](https://rustup.rs), затем:

```bash
cargo install --git https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
```

Бинарник попадёт в `~/.cargo/bin`, который rustup обычно добавляет в `PATH`.
Если после установки команда `terminal-session-proxy-manager` не находится, см.
[настройку PATH](#-настройка-path) ниже.

## 📥 Готовые сборки

Скачайте архив для своей платформы со
[страницы релизов](https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/releases).
Публикуются сборки для macOS (x86_64, arm64) и Linux (x86_64, arm64).

К каждому архиву прилагается файл `.sha256`. Проверьте перед установкой:

```bash
shasum -a 256 -c terminal-session-proxy-manager-macos-arm64.tar.gz.sha256

tar -xzf terminal-session-proxy-manager-macos-arm64.tar.gz
sudo mv terminal-session-proxy-manager /usr/local/bin/
```

На macOS Gatekeeper может поместить скачанный бинарник в карантин. Снять его:

```bash
xattr -d com.apple.quarantine /usr/local/bin/terminal-session-proxy-manager
```

---

## 🛠️ Сборка из исходников

```bash
git clone https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager

# Установка в ~/.cargo/bin
cargo install --path .

# Или просто сборка, бинарник останется в target/release/
cargo build --release
```

---

## 🔧 Настройка PATH

Если бинарник установлен, но не находится, добавьте его каталог в `PATH`.

Для установки через Cargo (`~/.zshrc` или `~/.bashrc`):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

При ручной установке в `/usr/local/bin` этот каталог обычно уже есть в `PATH`.
Проверить, какой именно бинарник используется:

```bash
which terminal-session-proxy-manager
```

---

## 🐚 Интеграция с shell (обязательно)

Независимо от способа установки добавьте скрипт инициализации в конфигурацию
оболочки. Без него команда `proxy` не существует, и ничто не сможет изменить
окружение вашей сессии — ни один процесс не может изменить родительскую
оболочку.

**Zsh** (`~/.zshrc`):

```bash
eval "$(terminal-session-proxy-manager init zsh)"
```

**Bash** (`~/.bashrc`):

```bash
eval "$(terminal-session-proxy-manager init bash)"
```

Перезапустите терминал или выполните `source ~/.zshrc`. Подробности и полный
список функций — в [SHELL_INTEGRATION.ru.md](SHELL_INTEGRATION.ru.md).

---

## 🔍 Проверка установки

```bash
terminal-session-proxy-manager --version
type proxy          # должна определиться shell-функция
proxy config path   # покажет используемый config.json
proxy status
```

---

## 🗑️ Удаление

```bash
# Homebrew
brew uninstall terminal-session-proxy-manager

# Cargo
cargo uninstall terminal-session-proxy-manager

# Ручная установка
sudo rm /usr/local/bin/terminal-session-proxy-manager
```

Затем удалите строку `eval "$(... init ...)"` из rc-файла оболочки.

Конфигурация намеренно остаётся на месте. Если она больше не нужна:

```bash
# macOS
rm -rf ~/Library/Application\ Support/terminal-session-proxy-manager
# Linux
rm -rf ~/.config/terminal-session-proxy-manager
```
