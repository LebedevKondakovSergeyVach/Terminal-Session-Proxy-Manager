# ==========================================================
# PROXY CLI — BASH INTEGRATION
# ==========================================================

proxy() {
    if command -v terminal-session-proxy-manager >/dev/null 2>&1; then
        if [ "$1" = "on" ] || [ "$1" = "off" ]; then
            eval "$(terminal-session-proxy-manager env "$1")"
        elif [ "$1" = "use" ]; then
            terminal-session-proxy-manager profile use "$2"
            [ -n "$ALL_PROXY" ] && eval "$(terminal-session-proxy-manager env on)"
        else
            terminal-session-proxy-manager "$@"
            if [ -f "$HOME/.terminal-session-proxy-manager-eval" ]; then
                if [ -f "$HOME/.terminal-session-proxy-manager-debug-enabled" ]; then
                    echo "[DEBUG-SHELL] Found $HOME/.terminal-session-proxy-manager-eval. Content:"
                    cat "$HOME/.terminal-session-proxy-manager-eval"
                    echo
                fi
                eval "$(cat "$HOME/.terminal-session-proxy-manager-eval")"
                rm -f "$HOME/.terminal-session-proxy-manager-eval"
            else
                if [ -f "$HOME/.terminal-session-proxy-manager-debug-enabled" ]; then
                    echo "[DEBUG-SHELL] $HOME/.terminal-session-proxy-manager-eval not found after terminal-session-proxy-manager exit."
                fi
            fi
        fi
    else
        echo "❌ terminal-session-proxy-manager бинарник не найден в PATH"
    fi
}

proxy_on() { eval "$(terminal-session-proxy-manager env on)"; }
proxy_off() { eval "$(terminal-session-proxy-manager env off)"; }
proxy_toggle() { if [ -n "$ALL_PROXY" ]; then proxy_off; else proxy_on; fi; }
proxy_status() { terminal-session-proxy-manager status "$@"; }
proxy_ping() { terminal-session-proxy-manager ping "$@"; }
proxy_diagnose() { terminal-session-proxy-manager diagnose "$@"; }
proxy_run() { terminal-session-proxy-manager run -- "$@"; }

# Предопределенные функции совместимости
proxy_set_throne() { proxy use throne; }
proxy_set_v2ray() { proxy use v2ray; }
proxy_set_port() {
    if [ -z "$1" ]; then
        echo "❌ Укажите порт. Пример: proxy_set_port 9050"
        return 1
    fi
    terminal-session-proxy-manager profile set custom --name "Custom ($1)" --port "$1"
    eval "$(terminal-session-proxy-manager env on)"
}
