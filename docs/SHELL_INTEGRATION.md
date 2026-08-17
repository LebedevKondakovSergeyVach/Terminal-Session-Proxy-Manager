# 🐚 Shell Integration

## Zsh (`~/.zshrc`)
```zsh
eval "$(terminal-session-proxy-manager init zsh)"
```
> Compatible with Powerlevel10k Instant Prompt (no unwanted output during terminal launch).

## Bash (`~/.bashrc`)
```bash
eval "$(terminal-session-proxy-manager init bash)"
```

---

## Available Shell Functions

| Command | Description |
| :--- | :--- |
| `proxy on` | Enable proxy for current shell session |
| `proxy off` | Disable proxy for current shell session |
| `proxy toggle` | Toggle proxy state (on/off) |
| `proxy status` | Check network status |
| `proxy use <key>` | Change active profile |
| `proxy ping` | Measure latency to endpoints |
| `proxy diagnose` | Socket & API diagnostics |
| `proxy run -- <cmd>` | Execute single command through proxy |
| `prompt_proxy_status` | Output proxy indicator for shell prompt segment |
