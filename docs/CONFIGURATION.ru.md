# ⚙️ Конфигурация

Два JSON-файла. Точные используемые пути покажут `proxy config path` и
`proxy settings path`.

## Расположение файлов

| ОС | Каталог |
| :--- | :--- |
| macOS | `~/Library/Application Support/terminal-session-proxy-manager/` |
| Linux | `~/.config/terminal-session-proxy-manager/` (учитывает `XDG_CONFIG_HOME`) |

### Порядок разрешения

**`config.json`**

1. `--config-file <ПУТЬ>`
2. `TSPM_CONFIG`
3. `config_path` из `settings.json` — относительный путь разрешается
   относительно каталога с `settings.json`, а не текущего рабочего каталога
4. Системный каталог конфигурации выше

**`settings.json`**

1. `--settings-file <ПУТЬ>`
2. `TSPM_SETTINGS`
3. Системный каталог конфигурации выше
4. `./settings.json` в текущем рабочем каталоге

Рабочий каталог намеренно стоит последним. `settings.json` — распространённое
имя файла, и если бы любой каталог, в котором вы оказались, имел приоритет над
вашей собственной конфигурацией, поведение программы зависело бы от места
запуска.

Оба файла создаются со значениями по умолчанию при первом запуске.
**Существующий файл с некорректным JSON не перезаписывается** — программа
сообщает об ошибке в stderr и оставляет файл нетронутым, поэтому опечатка не
уничтожит ваши профили.

---

## 1. `settings.json`

```json
{
  "config_path": null,
  "lang": "ru"
}
```

| Поле | Тип | Описание |
| :--- | :--- | :--- |
| `config_path` | строка или null | Путь к своему `config.json`. `~` раскрывается. `null` — системный каталог конфигурации. |
| `lang` | строка | Язык интерфейса: `"ru"` или `"en"`. По умолчанию `"ru"`. |

Изменяется командами `proxy settings set --config-path ~/proxies.json` или
`proxy lang en`.

---

## 2. `config.json`

```json
{
  "active_profile": "work",
  "profiles": {
    "work": {
      "name": "Рабочий SOCKS",
      "host": "127.0.0.1",
      "port": 1080,
      "protocol": "socks5"
    },
    "lab": {
      "name": "Лабораторный HTTP",
      "host": "127.0.0.1",
      "port": 8080,
      "protocol": "http"
    }
  },
  "ping_targets": [
    { "name": "Google", "url": "https://www.google.com" },
    { "name": "GitHub", "url": "https://github.com" }
  ],
  "diagnose_endpoints": [
    { "name": "GitHub API", "url": "https://api.github.com" }
  ],
  "geo_apis": [
    "https://ifconfig.co/json",
    "http://ip-api.com/json"
  ],
  "ipv4_api": "https://api4.ipify.org",
  "ipv6_api": "https://api6.ipify.org",
  "health_check_url": "https://www.gstatic.com/generate_204",
  "speedtest_url": "https://speed.cloudflare.com/__down?bytes=2097152"
}
```

Полный актуальный пример поставляется в файле
[`configs/config.default.json`](../configs/config.default.json).

### Поля

| Поле | Тип | Используется | Описание |
| :--- | :--- | :--- | :--- |
| `active_profile` | строка | везде | Ключ активной записи в `profiles` |
| `profiles` | объект | везде | Ключ профиля → объект профиля |
| `ping_targets` | массив | `ping`, `benchmark`, `best` | Эндпоинты для замера задержки |
| `diagnose_endpoints` | массив | `diagnose` | Эндпоинты для проверки доступности |
| `geo_apis` | массив | `status`, `dash` | JSON API геолокации, перебираются по порядку до первого ответа |
| `ipv4_api` | строка | `status` | Текстовый эндпоинт, возвращающий внешний IPv4 |
| `ipv6_api` | строка | `status` | Текстовый эндпоинт, возвращающий внешний IPv6 |
| `health_check_url` | строка | `monitor` | Проверяется для оценки работоспособности прокси. Лучше указывать адрес, отвечающий `204`. |
| `speedtest_url` | строка | `speedtest` | Загружается для замера скорости |

Все поля необязательны. Конфигурация без какого-либо из них — в том числе
созданная старой версией — загрузится, подставив значения по умолчанию.

### Объект профиля

| Поле | Тип | Описание |
| :--- | :--- | :--- |
| `name` | строка | Отображаемое имя |
| `host` | строка | Имя хоста или IP. Допустимы только буквы, цифры, `.`, `-`, `_`, `:`, `[`, `]`. |
| `port` | число | 1–65535 |
| `protocol` | строка | `http`, `https`, `socks4`, `socks4a`, `socks5`, `socks5h`. По умолчанию `socks5`. |

Профили проверяются при каждой записи — и через `profile set`, и через
`import`. Некорректный профиль отклоняется, а не сохраняется, поскольку его
значения попадают в URL прокси и в команды shell.

> **Примечание при обновлении с 2.1.x:** неиспользуемое поле верхнего уровня
> `enabled` удалено. Существующие файлы продолжают работать — поле просто
> игнорируется.
