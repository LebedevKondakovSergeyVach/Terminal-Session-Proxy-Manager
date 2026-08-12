# ⚙️ Configuration

## 1. `settings.json`
Specifies global application settings and language preference. Resolved in order: `~/.config/proxy-cli/settings.json` (or OS config dir) -> `./settings.json`.

```json
{
  "config_path": null,
  "lang": "ru"
}
```

- `"config_path"`: Optional path to custom configuration file (`null` uses default global `config.json`).
- `"lang"`: Interface language code (`"ru"` or `"en"`).

---

## 2. `config.json`
Main configuration file containing profiles, targets, and endpoints.

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

## 📋 Configuration Fields Schema

| Field | Type | Description |
| :--- | :--- | :--- |
| `active_profile` | String | Currently active key from `profiles` map |
| `profiles` | Map | Map of key-value profile objects |
| `ping_targets` | Array | Targets probed during `proxy ping` |
| `diagnose_endpoints` | Array | Target URLs checked by `proxy diagnose` |
| `geo_apis` | Array | IP / Location resolver JSON APIs |
