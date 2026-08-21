# 📖 Command Reference

Commands that change your current shell (`on`, `off`, `use`, `switch`, `best`)
must go through the `proxy` shell function installed by
[shell integration](SHELL_INTEGRATION.md). Everything else works with the full
binary name as well.

## Global options

Accepted by every subcommand, before or after it.

| Option | Environment variable | Purpose |
| :--- | :--- | :--- |
| `--config-file <PATH>` | `TSPM_CONFIG` | Use a specific `config.json` |
| `--settings-file <PATH>` | `TSPM_SETTINGS` | Use a specific `settings.json` |
| `--lang <ru\|en>` | `TSPM_LANG` | Interface language for this run only |
| `-h`, `--help` | | Help for the command |
| `-V`, `--version` | | Version |

`NO_COLOR` (set to any non-empty value) disables coloured output.

```bash
# Keep work and personal proxies in separate files
TSPM_CONFIG=~/work-proxies.json proxy best
terminal-session-proxy-manager --config-file ~/work-proxies.json profile list
```

---

## 1. Session control (`on`, `off`, `env`)

```bash
proxy on                     # Export proxy variables into this shell
proxy off                    # Unset them
terminal-session-proxy-manager env on   # Print the statements without applying them
```

`proxy on` is shorthand for evaluating the output of `env on`. Both set every
variable in both letter cases (`http_proxy` and `HTTP_PROXY`, and so on) plus
`GRADLE_OPTS` and `JAVA_TOOL_OPTIONS`, because different tools read different
names. `off` unsets exactly the same set.

Values are POSIX-quoted, so a host containing shell syntax is inert.

---

## 2. Network status (`status`)

```bash
proxy status         # Proxy state, IPv4, IPv6, and location
proxy status --json  # The same, machine-readable
```

The JSON form is stable enough to script against:

```bash
proxy status --json | jq -r .ipv4
```

---

## 3. Profiles (`profile`, `use`, `switch`)

```bash
proxy profile list                 # Every profile, with the active one marked
proxy switch                       # Interactive arrow-key picker
proxy use work                     # Switch by key

proxy profile set work \
  --name "Work SOCKS" \
  --host 127.0.0.1 \
  --port 1080 \
  --protocol socks5                # Create or update, then make it active

proxy profile remove work          # Delete
```

`--protocol` accepts `http`, `https`, `socks4`, `socks4a`, `socks5`, `socks5h`.
Omitted fields keep their current value on an existing profile.

Profiles are validated before being saved: the host must be a plausible hostname
or IP, the port must be non-zero, and the protocol must be supported. An unknown
profile key exits non-zero, so this works as expected:

```bash
proxy use work || proxy best
```

### Importing

```bash
proxy import ./proxies.json
proxy import https://example.com/subscription.txt
```

Four input shapes are recognised: a full `config.json`, a bare map of profile
key to profile, an array of profiles, or a newline-separated list of proxy URLs
(`socks5://host:port`, `#` comments allowed). Invalid entries are reported and
skipped; the rest still import.

---

## 4. Measurement (`benchmark`, `best`, `ping`, `speedtest`)

```bash
proxy benchmark            # Latency and availability of every profile
proxy best                 # Benchmark, then switch to the fastest
proxy ping                 # Latency to the endpoints in config.json
proxy ping --timeout 2000  # With a 2 s timeout
proxy speedtest            # Real download throughput
```

`benchmark` probes every profile against every ping target concurrently.
Unreachable profiles sort last and are shown as a timeout rather than a number.

---

## 5. Dashboard (`dash`)

```bash
proxy dash
```

| Key | Action |
| :--- | :--- |
| `↑` `↓` / `k` `j` | Move the selection |
| `Space` | Preview a profile without leaving |
| `Enter` | Apply the profile and exit, updating the shell |
| `b` | Switch to the fastest profile |
| `i` | Import from a URL or file |
| `e` | Open `config.json` in `$VISUAL` / `$EDITOR` |
| `s` | Benchmark everything and sort by latency |
| `1` `2` | Profiles tab / config tab |
| `q`, `Esc` | Quit |

For `Enter` to update your shell you need the `proxy_dash` function from the
shell integration; run `proxy dash`, not the bare binary.

---

## 6. Diagnostics (`diagnose`, `monitor`)

```bash
proxy diagnose  # Local socket, session variables, and endpoint reachability
proxy monitor   # Health check; switches to the fastest alternative on failure
```

`monitor` probes `health_check_url` from `config.json` through the proxy that is
currently in your environment. If it fails, it benchmarks the other profiles and
switches to the best reachable one.

---

## 7. Running one command (`run`)

```bash
proxy run curl https://example.com
proxy run -- curl -sS https://example.com
proxy run npm install
```

Runs a single command with the proxy variables applied, leaving your shell
untouched. The child's exit code is propagated, so `proxy run ... && next-step`
behaves as expected. Flags are passed through; `--` is optional but harmless.

---

## 8. Git integration (`git`)

```bash
proxy git status  # Show the current global git proxy
proxy git on      # Point git at the active profile
proxy git off     # Remove it
```

Writes `http.proxy` and `https.proxy` in your **global** git config. Unlike the
session commands this persists until you run `git off`.

---

## 9. Exporting (`export`)

```bash
proxy export envfile > .env
proxy export docker    # --build-arg flags
proxy export curl      # -x flag
```

Exits non-zero when no profile is active, so a redirect cannot silently produce
an empty file.

---

## 10. Configuration (`config`, `settings`, `lang`)

```bash
proxy config path        # Which config.json is in use
proxy config show        # Print it
proxy settings path
proxy settings show
proxy settings set --config-path ~/proxies.json
proxy lang en            # Persist the interface language
```

---

## 11. Shell setup (`init`, `completions`, `prompt`, `debug`)

```bash
terminal-session-proxy-manager init zsh          # Integration script
terminal-session-proxy-manager completions zsh   # Completions only
proxy prompt                                     # Prompt indicator
proxy debug on                                   # Log the shell hand-off
```

`completions` supports `zsh`, `bash`, `fish` and `powershell`. `init` already
includes completions, so you only need one of the two.

Turn `debug on` when `proxy dash` fails to update your shell; it logs the
hand-off to `~/.terminal-session-proxy-manager-debug.log`.
