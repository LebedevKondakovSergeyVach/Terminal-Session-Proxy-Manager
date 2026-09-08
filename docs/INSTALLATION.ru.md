# 📦 Установка (macOS и Linux)

<!--site:tabs-->

## Homebrew

Самый простой способ для macOS и Linux:

```bash title="Terminal" frame="terminal"
brew install LebedevKondakovSergeyVach/tap/terminal-session-proxy-manager
```

Обновление в дальнейшем:

```bash title="Terminal" frame="terminal"
brew upgrade terminal-session-proxy-manager
```

## Cargo

Требуется Rust **1.88 или новее** (проект использует редакцию 2024 года).
Установите тулчейн с [rustup.rs](https://rustup.rs), затем выполните:

```bash title="Terminal" frame="terminal"
cargo install --git https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
```

Бинарник будет помещён в `~/.cargo/bin`, который rustup обычно сам добавляет в
ваш `PATH`. Если после этого утилита не найдена, см. раздел
[Настройка PATH](#-настройка-path) ниже.

## Binary

Скачайте архив для вашей платформы со
[страницы релизов](https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/releases).
Сборки публикуются для macOS (x86_64, arm64) и Linux (x86_64, arm64).

Каждый архив сопровождается файлом `.sha256`. Проверьте его перед установкой:

```bash title="Terminal" frame="terminal"
shasum -a 256 -c terminal-session-proxy-manager-macos-arm64.tar.gz.sha256

tar -xzf terminal-session-proxy-manager-macos-arm64.tar.gz
sudo mv terminal-session-proxy-manager /usr/local/bin/
```

<!--site:/tabs-->

В macOS Gatekeeper может поместить скачанный файл в карантин. Чтобы снять его:

```bash title="Terminal" frame="terminal"
xattr -d com.apple.quarantine /usr/local/bin/terminal-session-proxy-manager
```

---

## 🛠️ Сборка из исходников

```bash title="Terminal" frame="terminal"
git clone https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager

# Установить в ~/.cargo/bin
cargo install --path .

# Или просто собрать (бинарник останется в target/release/)
cargo build --release
```

---

## 🔧 Настройка PATH

Если программа установлена, но командный интерпретатор её не находит, добавьте
директорию с ней в `PATH`.

При установке через Cargo (`~/.zshrc` или `~/.bashrc`):

```bash title="~/.zshrc" frame="code"
export PATH="$HOME/.cargo/bin:$PATH"
```

При ручной установке в `/usr/local/bin` эта директория обычно уже есть в `PATH`.
Проверьте, какой именно бинарник используется:

```bash title="Terminal" frame="terminal"
which terminal-session-proxy-manager
```

---

## 🐚 Интеграция с shell (обязательно)

<!--site:aside type="caution"-->
Независимо от способа установки, вам необходимо прописать скрипт инициализации в конфигурацию вашей оболочки, чтобы команда `proxy on` могла изменять переменные окружения. См. [Интеграция с оболочкой](SHELL_INTEGRATION.ru.md).
<!--site:/aside-->

**Zsh** (`~/.zshrc`):

```bash title="~/.zshrc" frame="code"
eval "$(terminal-session-proxy-manager init zsh)"
```

**Bash** (`~/.bashrc`):

```bash title="~/.bashrc" frame="code"
eval "$(terminal-session-proxy-manager init bash)"
```

Перезапустите терминал или выполните `source ~/.zshrc` для применения изменений.
Подробности и полный список команд см. в
[SHELL_INTEGRATION.ru.md](SHELL_INTEGRATION.ru.md).

---

## 🔍 Проверка установки

```bash title="Terminal" frame="terminal"
terminal-session-proxy-manager --version
type proxy          # должен сообщить, что это shell-функция
proxy config path   # покажет, какой config.json используется
proxy status
```

---

## 🗑️ Удаление

```bash title="Terminal" frame="terminal"
# Homebrew
brew uninstall terminal-session-proxy-manager

# Cargo
cargo uninstall terminal-session-proxy-manager

# Ручная установка
sudo rm /usr/local/bin/terminal-session-proxy-manager
```

Затем удалите строку `eval "$(... init ...)"` из файла конфигурации вашей
оболочки (rc-файла).

Настройки намеренно не удаляются. Если хотите удалить и их:

```bash title="Terminal" frame="terminal"
# macOS
rm -rf ~/Library/Application\ Support/terminal-session-proxy-manager
# Linux
rm -rf ~/.config/terminal-session-proxy-manager
```
