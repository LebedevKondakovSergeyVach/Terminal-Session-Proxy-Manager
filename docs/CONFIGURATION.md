# ⚙️ Configuration

Two JSON files. `proxy config path` and `proxy settings path` print exactly
which ones are in use.

## Where the files live

| OS | Directory |
| :--- | :--- |
| macOS | `~/Library/Application Support/terminal-session-proxy-manager/` |
| Linux | `~/.config/terminal-session-proxy-manager/` (respects `XDG_CONFIG_HOME`) |

### Resolution order

**`config.json`**

1. `--config-file <PATH>`
2. `TSPM_CONFIG`
3. `config_path` in `settings.json` — a relative value is resolved against the
   directory holding `settings.json`, not your working directory
4. The OS config directory above

**`settings.json`**

1. `--settings-file <PATH>`
2. `TSPM_SETTINGS`
3. The OS config directory above
4. `./settings.json` in the working directory

The working-directory entry is last on purpose. `settings.json` is a common
filename, and letting any directory you happen to be in outrank your own
configuration would make the tool behave differently depending on where it was
run from.

Both files are created with defaults on first run. **A file that exists but
contains invalid JSON is reported on stderr and left untouched** — it is never
overwritten, so a typo cannot cost you your profiles.

---

## 1. `settings.json`

```json
{
  "config_path": null,
  "lang": "ru"
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `config_path` | string or null | Path to a custom `config.json`. `~` is expanded. `null` uses the OS config directory. |
| `lang` | string | Interface language: `"ru"` or `"en"`. Defaults to `"ru"`. |

Edit it with `proxy settings set --config-path ~/proxies.json` or `proxy lang en`.

---

## 2. `config.json`

```json
{
  "active_profile": "work",
  "profiles": {
    "work": {
      "name": "Work SOCKS",
      "host": "127.0.0.1",
      "port": 1080,
      "protocol": "socks5"
    },
    "lab": {
      "name": "Lab HTTP",
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

A complete, current example ships as
[`configs/config.default.json`](../configs/config.default.json).

### Fields

| Field | Type | Used by | Description |
| :--- | :--- | :--- | :--- |
| `active_profile` | string | everything | Key of the active entry in `profiles` |
| `profiles` | map | everything | Profile key → profile object |
| `ping_targets` | array | `ping`, `benchmark`, `best` | Endpoints probed for latency |
| `diagnose_endpoints` | array | `diagnose` | Endpoints checked for reachability |
| `geo_apis` | array | `status`, `dash` | JSON IP/geolocation APIs, tried in order until one answers |
| `ipv4_api` | string | `status` | Plain-text endpoint returning your external IPv4 |
| `ipv6_api` | string | `status` | Plain-text endpoint returning your external IPv6 |
| `health_check_url` | string | `monitor` | Probed to decide whether the proxy still works. Use something that answers `204`. |
| `speedtest_url` | string | `speedtest` | Downloaded to measure throughput |

Every field is optional. A config missing any of them — including one written by
an older version — loads with the built-in default for whatever is absent.

### Profile object

| Field | Type | Description |
| :--- | :--- | :--- |
| `name` | string | Display name |
| `host` | string | Hostname or IP. Letters, digits, `.`, `-`, `_`, `:`, `[`, `]` only. |
| `port` | number | 1–65535 |
| `protocol` | string | `http`, `https`, `socks4`, `socks4a`, `socks5`, `socks5h`. Defaults to `socks5`. |

Profiles are validated whenever they are written, whether by `profile set` or by
`import`. An invalid profile is rejected rather than saved, because the values
end up in a proxy URL and in shell statements.

> **Note for users upgrading from 2.1.x:** the unused top-level `enabled` field
> was removed. Existing files keep working; the field is simply ignored.
