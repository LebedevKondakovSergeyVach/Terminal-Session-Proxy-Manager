<p align="center">
  🇬🇧 <b>English</b> | 🇷🇺 <a href="README.ru.md">Русский</a>
</p>

# ⚡ Terminal Session Proxy Manager

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](README.md)

Universal, high-performance terminal session proxy management toolkit in **Rust** for session control, IP/location diagnostics, speed benchmarking, and environment variable management on **macOS** and **Linux** (Zsh / Bash).

## ✨ Features

- **Blazing Fast**: Written in pure Rust, offering exceptional performance with minimal overhead.
- **Interactive TTY**: Select profiles interactively using `dialoguer`.
- **Auto-healing**: Real-time proxy monitoring with automatic failover to the fastest available node.
- **Diagnostics**: Built-in ping and speedtest functionality to measure actual bandwidth.
- **Universal Export**: One-command export for Docker, cURL, Git, and environment variables.
- **Localization**: Full bilingual support (English and Russian).

---

## ⚡ Quick Start

### 🍏 macOS
```bash
cargo build --release
cp target/release/proxy-cli /opt/homebrew/bin/

# Add to ~/.zshrc:
eval "$(proxy-cli init zsh)"
```

### 🐧 Linux
```bash
cargo build --release
sudo cp target/release/proxy-cli /usr/local/bin/

# Add to ~/.bashrc (or ~/.zshrc):
eval "$(proxy-cli init bash)"
```

---

## 🚀 Commands

| Command | Description |
| :--- | :--- |
| `proxy status` | Check proxy status, IPv4, IPv6, and physical location |
| `proxy status --json` | Output network status in JSON format |
| `proxy on` | Enable proxy for current shell session |
| `proxy off` | Disable proxy for current shell session |
| `proxy git <on/off/status>` | Manage global Git proxy configuration |
| `proxy export <docker/curl/env>` | Export proxy settings for Docker, cURL, or `.env` |
| `proxy speedtest` | Benchmark real download throughput in MB/s |
| `proxy monitor` | Monitor health & auto-fallback on connection failure |
| `proxy lang <ru/en>` | Switch interface language (`ru`, `en`) |
| `proxy use <profile>` | Select active proxy profile (`throne`, `v2ray`) |
| `proxy dash` | Launch interactive TUI Dashboard (monitor, benchmark, switch) |
| `proxy switch` | Interactive arrow-key profile selector |
| `proxy benchmark` | Benchmark ping & availability of all profiles |
| `proxy best` | Automatically select the fastest proxy with lowest ping |
| `proxy import <source>` | Import proxy profiles from local JSON file or URL |
| `proxy ping` | Probe latency to target endpoints |
| `proxy diagnose` | Extended diagnostics for local sockets & HTTP APIs |
| `proxy run -- <cmd>` | Execute a single command through proxy |
| `proxy-cli completions zsh` | Generate auto-completion scripts |

---

## 📚 Documentation

- [📦 **Installation (macOS & Linux)**](docs/INSTALLATION.md) — Detailed build instructions & PATH setups.
- [🐚 **Shell Integration**](docs/SHELL_INTEGRATION.md) — Zsh, Bash, and prompt integration.
- [⚙️ **Configuration**](docs/CONFIGURATION.md) — `settings.json` and `config.json` schema.
- [📖 **Command Reference**](docs/USAGE.md) — Full subcommands reference guide.

---

## 📄 License

[GNU AGPLv3](LICENSE)
