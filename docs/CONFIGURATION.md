# ⚙️ Подробное Руководство по Конфигурации

Архитектура конфигурации **Proxy CLI** состоит из двух уровней:

1. **`settings.json`** — Файл настроек окружения, указывающий путь к активному конфигу.
2. **`config.json`** — Основной файл конфигурации с описанием профилей прокси, пинг-целей и эндпоинтов.

---

## 📄 1. Уровень 1: `settings.json`

Файл `settings.json` определяет, какой именно конфигурационный файл должна загружать утилита.

### Порядок поиска `settings.json`:
1. Текущий рабочий каталог: `./settings.json` или `./settings.local.json`.
2. Системная папка пользователя:
   - **macOS**: `~/Library/Application Support/proxy-cli/settings.json`
   - **Linux**: `~/.config/proxy-cli/settings.json`

### Структура `settings.json`:
```json
{
  "config_path": "./configs/config.throne-v2ray.json"
}
```

> [!NOTE]
> В поле `"config_path"` поддерживается относительный путь (относительно корневой директории или `settings.json`), тильда (`~`), а также абсолютные пути.

---

## 🛠️ 2. Уровень 2: `config.json`

Основной файл конфигурации полностью определяет поведение программы. Никакие имена сервисов или порты не зашиты в код программы!

### Схема `config.json`:
```json
{
  "active_profile": "throne",
  "enabled": false,
  "profiles": {
    "throne": {
      "name": "Throne",
      "host": "127.0.0.1",
      "port": 2080,
      "protocol": "socks5"
    },
    "v2ray": {
      "name": "v2rayN",
      "host": "127.0.0.1",
      "port": 10808,
      "protocol": "socks5"
    }
  },
  "ping_targets": [
    { "name": "Google API", "url": "https://generativelanguage.googleapis.com" },
    { "name": "GitHub", "url": "https://github.com" },
    { "name": "OpenAI API", "url": "https://api.openai.com" }
  ],
  "diagnose_endpoints": [
    { "name": "Gemini API", "url": "https://generativelanguage.googleapis.com" },
    { "name": "Cloud Code", "url": "https://daily-cloudcode-pa.googleapis.com" }
  ],
  "geo_apis": [
    "https://ifconfig.co/json",
    "http://ip-api.com/json"
  ]
}
```

---

## 📝 Описание полей

| Раздел / Поле | Тип | Описание |
| :--- | :--- | :--- |
| `active_profile` | String | Ключ текущего активного профиля из объекта `profiles` |
| `profiles` | Map | Словарь настроек доступных прокси-профилей |
| `profiles.<key>.name` | String | Название профиля (отображается в статусе) |
| `profiles.<key>.host` | String | Хост прокси (например: `127.0.0.1`) |
| `profiles.<key>.port` | Integer | Порт прокси (например: `2080`, `10808`) |
| `profiles.<key>.protocol` | String | Протокол (`socks5`, `http`) |
| `ping_targets` | Array | Список URL ресурсов для проверки задержки (команда `proxy ping`) |
| `diagnose_endpoints` | Array | Список ресурсов для расширенной диагностики (команда `proxy diagnose`) |
| `geo_apis` | Array | Список внешних JSON API для определения геолокации и IP |
