<p align="center">
  🇬🇧 <b>English</b> | 🇷🇺 <a href="README.ru.md">Русский</a>
</p>

# ⚡ Terminal Session Proxy Manager

![Project Banner](assets/banner_new.jpg)

[![CI](https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/actions/workflows/ci.yml/badge.svg)](https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](README.md)
![Views](https://komarev.com/ghpvc/?username=LebedevSergeyVach-proxy-cli&label=views&color=blue&style=flat)

Manage proxy profiles for your terminal session from one command, in **Rust**:
switch profiles, benchmark latency, diagnose connectivity, and control the
environment variables your tools actually read — on **macOS** and **Linux**
(Zsh / Bash).

## ✨ Features

- **Profiles, not exports.** Name your proxies once, then switch with a single
  command or an interactive picker.
- **Interactive TUI dashboard.** Live IP, geolocation, latency sparkline, and
  profile switching in one screen.
- **Pick the fastest automatically.** Benchmarks every profile in parallel and
  selects the best one, with auto-failover when the active proxy dies.
- **Diagnostics.** Latency probes, socket checks, and a real bandwidth test.
- **Exports everything.** One command for Docker, cURL, Git, `.env`, and JVM
  build tooling — with correct shell quoting.
- **Bilingual.** Full English and Russian interface.

![Interactive Dashboard](assets/proxy_dashboard_final.png)

---

## ⚡ Quick Start

### 📦 Installation

**Homebrew (macOS / Linux):**

```bash
brew install LebedevKondakovSergeyVach/tap/terminal-session-proxy-manager
```

**Cargo** (requires Rust 1.85+):

```bash
cargo install --git https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
```

**From source:**

```bash
git clone https://github.com/LebedevKondakovSergeyVach/Terminal-Session-Proxy-Manager.git
cd Terminal-Session-Proxy-Manager
cargo install --path .
```

### 🐚 Shell Integration

The binary cannot change the environment of the shell that launched it — no
process can. The `proxy` shell function bridges that gap, so add the init script
to your shell configuration:

**Zsh** (`~/.zshrc`):

```bash
eval "$(terminal-session-proxy-manager init zsh)"
```

**Bash** (`~/.bashrc`):

```bash
eval "$(terminal-session-proxy-manager init bash)"
```

Restart your terminal or run `source ~/.zshrc`. You now have `proxy on`,
`proxy off`, `proxy switch`, and tab completion.

### First run

```bash
proxy profile set home --name "Home" --host 127.0.0.1 --port 1080
proxy on
proxy status
```

---

## 🚀 Commands

Anything that changes your current shell goes through the `proxy` function;
everything else works with the full binary name too.

### Session

| Command | Description |
| :--- | :--- |
| `proxy on` | Enable the proxy for this shell session |
| `proxy off` | Disable it |
| `proxy status` | Proxy state, IPv4, IPv6, and physical location |
| `proxy status --json` | The same, as JSON |
| `proxy run -- <cmd>` | Run one command through the proxy, leaving the shell untouched |
| `proxy prompt` | Prompt indicator, for embedding in `PS1` / `PROMPT` |

### Profiles

| Command | Description |
| :--- | :--- |
| `proxy profile list` | List every profile |
| `proxy switch` | Interactive arrow-key picker |
| `proxy use <key>` | Switch to a profile by key |
| `proxy profile set <key> [--name N] [--host H] [--port P] [--protocol P]` | Add or update a profile |
| `proxy profile remove <key>` | Delete a profile |
| `proxy import <source>` | Import profiles from a JSON file or a subscription URL |

### Measurement

| Command | Description |
| :--- | :--- |
| `proxy dash` | Interactive TUI dashboard |
| `proxy benchmark` | Latency and availability of every profile |
| `proxy best` | Benchmark, then switch to the fastest |
| `proxy ping [--timeout MS]` | Latency to the endpoints in `config.json` |
| `proxy speedtest` | Real download throughput |
| `proxy monitor` | Health check with automatic failover |
| `proxy diagnose` | Socket and endpoint diagnostics |

### Integration

| Command | Description |
| :--- | :--- |
| `proxy git <on\|off\|status>` | Global Git proxy configuration |
| `proxy export <docker\|curl\|envfile>` | Export settings for other tools |
| `proxy env <on\|off>` | Print raw shell statements (what `proxy on` evaluates) |
| `... init <zsh\|bash>` | Print the shell integration script |
| `... completions <zsh\|bash\|fish\|powershell>` | Print a completion script |

### Configuration

| Command | Description |
| :--- | :--- |
| `proxy config path` | Path to the active `config.json` |
| `proxy config show` | Print the current configuration |
| `proxy settings path` | Path to `settings.json` |
| `proxy settings show` | Print global settings |
| `proxy lang <ru\|en>` | Switch interface language |
| `proxy debug <on\|off>` | Shell-integration debug logging |

### Global options

Available on every subcommand.

| Option | Environment variable | Purpose |
| :--- | :--- | :--- |
| `--config-file <PATH>` | `TSPM_CONFIG` | Use a specific `config.json` |
| `--settings-file <PATH>` | `TSPM_SETTINGS` | Use a specific `settings.json` |
| `--lang <ru\|en>` | `TSPM_LANG` | Language for this run only |

`NO_COLOR` is honoured. Useful for keeping several setups apart:

```bash
TSPM_CONFIG=~/work-proxies.json proxy best
```

---

## ⚙️ Configuration

Two files, both JSON:

- **`config.json`** — profiles and the endpoints used for testing.
- **`settings.json`** — interface language and an optional custom config path.

Both live in your OS configuration directory
(`~/Library/Application Support/terminal-session-proxy-manager` on macOS,
`~/.config/terminal-session-proxy-manager` on Linux). Run `proxy config path` to
see exactly which file is in use.

Resolution order for `config.json`: `--config-file` → `TSPM_CONFIG` →
`config_path` in `settings.json` → the OS config directory.

A config file that exists but contains invalid JSON is **reported and left
alone** — it is never overwritten with defaults.

See [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md) for the full schema, and
[`configs/config.default.json`](configs/config.default.json) for a complete
example.

---

## 📚 Documentation

- [📦 **Installation**](docs/INSTALLATION.md) — build instructions and PATH setup
- [🐚 **Shell Integration**](docs/SHELL_INTEGRATION.md) — Zsh, Bash, and prompt setup
- [⚙️ **Configuration**](docs/CONFIGURATION.md) — full `config.json` and `settings.json` schema
- [📖 **Command Reference**](docs/USAGE.md) — every subcommand and flag
- [🤝 **Contributing**](CONTRIBUTING.md) · [🔒 **Security**](SECURITY.md) · [📝 **Changelog**](CHANGELOG.md)

---

## 📄 License

[GNU AGPLv3](LICENSE)
