# ==========================================================
# PROXY CLI — ZSH INTEGRATION
# ==========================================================

proxy() {
    if command -v proxy-cli >/dev/null 2>&1; then
        if [[ "$1" == "on" || "$1" == "off" ]]; then
            eval "$(proxy-cli env "$1")"
        elif [[ "$1" == "use" ]]; then
            proxy-cli profile use "$2"
            [ -n "$ALL_PROXY" ] && eval "$(proxy-cli env on)"
        else
            proxy-cli "$@"
            if [ -f "$HOME/.proxy-cli-eval" ]; then
                eval "$(cat "$HOME/.proxy-cli-eval")"
                rm -f "$HOME/.proxy-cli-eval"
            fi
        fi
    else
        echo "❌ proxy-cli бинарник не найден в PATH"
    fi
}

proxy_on() { eval "$(proxy-cli env on)"; }
proxy_off() { eval "$(proxy-cli env off)"; }
proxy_toggle() { if [ -n "$ALL_PROXY" ]; then proxy_off; else proxy_on; fi; }
proxy_status() { proxy-cli status "$@"; }
proxy_ping() { proxy-cli ping "$@"; }
proxy_diagnose() { proxy-cli diagnose "$@"; }
proxy_run() { proxy-cli run -- "$@"; }
prompt_proxy_status() { proxy-cli prompt; }

# Предопределенные функции совместимости
proxy_set_throne() { proxy use throne; }
proxy_set_v2ray() { proxy use v2ray; }
proxy_set_port() {
    if [ -z "$1" ]; then
        echo "❌ Укажите порт. Пример: proxy_set_port 9050"
        return 1
    fi
    proxy-cli profile set custom --name "Custom ($1)" --port "$1"
    eval "$(proxy-cli env on)"
}
