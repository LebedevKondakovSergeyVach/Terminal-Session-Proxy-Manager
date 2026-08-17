# 📖 Command Reference

## 1. Network Status (`status`)
```bash
proxy status         # Print network status & location
proxy status --json  # Output status in JSON format
```

---

## 2. Interface Language (`lang`)
```bash
proxy lang en # Switch UI language to English
proxy lang ru # Switch UI language to Russian
```

---

## 3. Git Proxy Integration (`git`)
```bash
proxy git on     # Set global proxy for git clone / git push
proxy git off    # Unset global proxy in git config
proxy git status # Show current git proxy settings
```

---

## 4. Configuration Export (`export`)
```bash
proxy export docker  # Output --build-arg flags for Docker
proxy export curl    # Output -x proxy flags for cURL
proxy export envfile # Output .env file variables
```

---

## 5. Bandwidth Speed Test (`speedtest`)
```bash
proxy speedtest # Measure real download throughput in MB/s
```

---

## 6. Health Monitor & Auto-Heal (`monitor`)
```bash
proxy monitor # Probe socket health; auto-switch to fallback proxy on failure
```

---

## 7. Profile Management (`profile`)
```bash
terminal-session-proxy-manager profile list       # List all configured profiles
proxy switch                 # Interactive arrow-key profile selector
proxy use v2ray              # Switch active profile by key
proxy dash                   # Launch interactive TUI Dashboard (monitor, benchmark, switch)
proxy best                   # Auto-select profile with lowest latency
proxy benchmark              # Measure latency & availability of all profiles
proxy import <file/url>      # Import profiles from local JSON file or URL
```

---

## 8. Latency Probe (`ping`)
```bash
proxy ping                 # Probe ping latency to target endpoints
terminal-session-proxy-manager ping --timeout 2000 # Probe with 2000ms timeout
```

---

## 9. Network Diagnostics (`diagnose`)
```bash
proxy diagnose # Detailed local socket, environment variables, & API check
```

---

## 10. Execute Command Through Proxy (`run`)
```bash
proxy run -- curl https://api.openai.com
```

---

## 11. Shell Autocompletions (`completions`)
```bash
terminal-session-proxy-manager completions zsh > ~/.zsh/completions/_terminal-session-proxy-manager
```
