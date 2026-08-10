# ⚙️ Конфигурация

## 1. `settings.json`
Указывает путь к файлу конфигурации и предпочитаемый язык интерфейса. Ищется по путям `./settings.json` -> `./settings.local.json` -> `~/.config/proxy-cli/settings.json`.

```json
{
  "config_path": "./configs/config.throne-v2ray.json",
  "lang": "ru"
}
```

- `"config_path"`: Путь к целевому файлу конфигурации.
- `"lang"`: Код языка интерфейса (`"ru"` или `"en"`).

---

## 2. `config.json`
Основной файл конфигурации с описанием профилей и эндпоинтов.

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
    { "name": "GitHub", "url": "https://github.com" }
  ],
  "diagnose_endpoints": [
    { "name": "Gemini API", "url": "https://generativelanguage.googleapis.com" }
  ],
  "geo_apis": [
    "https://ifconfig.co/json"
  ]
}
```

---

## 📋 Поля конфигурации

| Поле | Тип | Описание |
| :--- | :--- | :--- |
| `active_profile` | String | Ключ активного профиля из `profiles` |
| `profiles` | Map | Объект настроек профилей |
| `ping_targets` | Array | Список URL для `proxy ping` |
| `diagnose_endpoints` | Array | Список URL для `proxy diagnose` |
| `geo_apis` | Array | JSON API для определения IP/гео |
