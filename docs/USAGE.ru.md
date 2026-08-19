# 📖 Справочник команд

Команды, изменяющие текущую сессию (`on`, `off`, `use`, `switch`, `best`),
работают только через shell-функцию `proxy`, которую устанавливает
[интеграция с shell](SHELL_INTEGRATION.ru.md). Остальные доступны и по полному
имени бинарника.

## Глобальные опции

Принимаются любой подкомандой, до или после неё.

| Опция | Переменная окружения | Назначение |
| :--- | :--- | :--- |
| `--config-file <ПУТЬ>` | `TSPM_CONFIG` | Использовать конкретный `config.json` |
| `--settings-file <ПУТЬ>` | `TSPM_SETTINGS` | Использовать конкретный `settings.json` |
| `--lang <ru\|en>` | `TSPM_LANG` | Язык интерфейса только для этого запуска |
| `-h`, `--help` | | Справка по команде |
| `-V`, `--version` | | Версия |

Переменная `NO_COLOR` (любое непустое значение) отключает цветной вывод.

```bash
# Рабочие и личные прокси в разных файлах
TSPM_CONFIG=~/work-proxies.json proxy best
terminal-session-proxy-manager --config-file ~/work-proxies.json profile list
```

---

## 1. Управление сессией (`on`, `off`, `env`)

```bash
proxy on                     # Экспортировать переменные прокси в эту сессию
proxy off                    # Убрать их
terminal-session-proxy-manager env on   # Показать команды, не применяя их
```

`proxy on` — это выполнение вывода `env on`. Обе команды задают каждую
переменную в обоих регистрах (`http_proxy` и `HTTP_PROXY` и так далее), а также
`GRADLE_OPTS` и `JAVA_TOOL_OPTIONS`, потому что разные инструменты читают разные
имена. `off` снимает ровно тот же набор.

Значения экранируются по правилам POSIX, поэтому хост с shell-синтаксисом
безопасен.

---

## 2. Состояние сети (`status`)

```bash
proxy status         # Состояние прокси, IPv4, IPv6 и геолокация
proxy status --json  # То же самое в машиночитаемом виде
```

JSON пригоден для скриптов:

```bash
proxy status --json | jq -r .ipv4
```

---

## 3. Профили (`profile`, `use`, `switch`)

```bash
proxy profile list                 # Все профили, активный отмечен
proxy switch                       # Интерактивный выбор стрелками
proxy use work                     # Переключение по ключу

proxy profile set work \
  --name "Рабочий SOCKS" \
  --host 127.0.0.1 \
  --port 1080 \
  --protocol socks5                # Создать или изменить и сделать активным

proxy profile remove work          # Удалить
```

`--protocol` принимает `http`, `https`, `socks4`, `socks4a`, `socks5`,
`socks5h`. Не указанные поля у существующего профиля сохраняют прежние значения.

Профили проверяются перед сохранением: хост должен быть корректным именем или
IP, порт — ненулевым, протокол — поддерживаемым. Неизвестный ключ профиля даёт
ненулевой код возврата, поэтому такая конструкция работает как ожидается:

```bash
proxy use work || proxy best
```

### Импорт

```bash
proxy import ./proxies.json
proxy import https://example.com/subscription.txt
```

Распознаются четыре формата: полный `config.json`, простое соответствие ключа
профилю, массив профилей и список прокси-URL по одному в строке
(`socks5://host:port`, комментарии через `#`). Некорректные записи пропускаются
с сообщением, остальные импортируются.

---

## 4. Измерения (`benchmark`, `best`, `ping`, `speedtest`)

```bash
proxy benchmark            # Задержка и доступность всех профилей
proxy best                 # Замерить и переключиться на самый быстрый
proxy ping                 # Задержка до эндпоинтов из config.json
proxy ping --timeout 2000  # С таймаутом 2 с
proxy speedtest            # Реальная скорость загрузки
```

`benchmark` проверяет каждый профиль по каждой цели параллельно. Недоступные
профили оказываются в конце списка и отображаются как таймаут, а не числом.

---

## 5. Дашборд (`dash`)

```bash
proxy dash
```

| Клавиша | Действие |
| :--- | :--- |
| `↑` `↓` / `k` `j` | Перемещение по списку |
| `Space` | Предпросмотр профиля без выхода |
| `Enter` | Применить профиль, выйти и обновить сессию |
| `b` | Переключиться на самый быстрый |
| `i` | Импорт из URL или файла |
| `e` | Открыть `config.json` в `$VISUAL` / `$EDITOR` |
| `s` | Замерить все профили и отсортировать по задержке |
| `1` `2` | Вкладка профилей / вкладка конфигурации |
| `q`, `Esc` | Выход |

Чтобы `Enter` обновил вашу сессию, нужна функция `proxy_dash` из интеграции с
shell — запускайте `proxy dash`, а не бинарник напрямую.

---

## 6. Диагностика (`diagnose`, `monitor`)

```bash
proxy diagnose  # Локальный сокет, переменные сессии и доступность эндпоинтов
proxy monitor   # Проверка состояния с переключением на лучший при сбое
```

`monitor` проверяет `health_check_url` из `config.json` через тот прокси,
который сейчас установлен в вашем окружении. При сбое замеряет остальные
профили и переключается на лучший доступный.

---

## 7. Запуск одной команды (`run`)

```bash
proxy run curl https://example.com
proxy run -- curl -sS https://example.com
proxy run npm install
```

Выполняет одну команду с переменными прокси, не меняя вашу сессию. Код возврата
дочернего процесса передаётся наружу, поэтому `proxy run ... && next-step`
работает корректно. Флаги передаются как есть; `--` необязателен.

---

## 8. Интеграция с Git (`git`)

```bash
proxy git status  # Показать текущий глобальный прокси git
proxy git on      # Направить git на активный профиль
proxy git off     # Убрать
```

Записывает `http.proxy` и `https.proxy` в **глобальный** конфиг git. В отличие
от команд сессии, это сохраняется до выполнения `git off`.

---

## 9. Экспорт (`export`)

```bash
proxy export envfile > .env
proxy export docker    # Флаги --build-arg
proxy export curl      # Флаг -x
```

Завершается с ошибкой, если активного профиля нет, поэтому перенаправление не
создаст пустой файл незаметно.

---

## 10. Конфигурация (`config`, `settings`, `lang`)

```bash
proxy config path        # Какой config.json используется
proxy config show        # Показать его
proxy settings path
proxy settings show
proxy settings set --config-path ~/proxies.json
proxy lang ru            # Сохранить язык интерфейса
```

---

## 11. Настройка shell (`init`, `completions`, `prompt`, `debug`)

```bash
terminal-session-proxy-manager init zsh          # Скрипт интеграции
terminal-session-proxy-manager completions zsh   # Только автодополнение
proxy prompt                                     # Индикатор для приглашения
proxy debug on                                   # Логировать обмен с shell
```

`completions` поддерживает `zsh`, `bash`, `fish` и `powershell`. `init` уже
включает автодополнение, поэтому достаточно чего-то одного.

Включайте `debug on`, если `proxy dash` не обновляет сессию: обмен пишется в
`~/.terminal-session-proxy-manager-debug.log`.
