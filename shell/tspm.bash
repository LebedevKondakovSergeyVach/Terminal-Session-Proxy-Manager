# Bash integration for Terminal Session Proxy Manager.
# Generated from: tspm init bash
# Preferred startup: eval "$(tspm init bash)"
# Completions omitted; use tspm completions bash.

# tspm Bash Integration
proxy() {
    local _tspm_rc _tspm_exports
    if command -v tspm >/dev/null 2>&1; then
        if [ "$1" = "on" ] || [ "$1" = "off" ]; then
            _tspm_exports="$(tspm env "$1")" || return $?
            eval "$_tspm_exports"
        elif [ "$1" = "use" ] || [ "$1" = "switch" ] || [ "$1" = "best" ]; then
            case "$1" in
                use)    tspm profile use "$2" ;;
                switch) tspm profile select ;;
                best)   tspm profile best ;;
            esac
            # Capture first: the re-apply test used to become the function's
            # own status, so `proxy use work` reported failure on success when
            # the proxy was off, and success on failure when it was on.
            _tspm_rc=$?
            if [ $_tspm_rc -eq 0 ] && [ -n "$ALL_PROXY" ]; then
                _tspm_exports="$(tspm env on)" || return $?
                eval "$_tspm_exports"
            fi
            return $_tspm_rc
        else
            tspm "$@"
            _tspm_rc=$?
            if [ -f "$HOME/.tspm-eval" ]; then
                if [ -f "$HOME/.tspm-debug-enabled" ]; then
                    echo "[DEBUG-SHELL] Found $HOME/.tspm-eval. Content:"
                    cat "$HOME/.tspm-eval"
                    echo
                fi
                eval "$(cat "$HOME/.tspm-eval")"
                rm -f "$HOME/.tspm-eval"
            else
                if [ -f "$HOME/.tspm-debug-enabled" ]; then
                    echo "[DEBUG-SHELL] $HOME/.tspm-eval not found after tspm exit."
                fi
            fi
            return $_tspm_rc
        fi
    else
        echo "❌ tspm binary not found in PATH"
        return 127
    fi
}

proxy_on() { proxy on; }
proxy_off() { proxy off; }
proxy_toggle() { if [ -n "$ALL_PROXY" ]; then proxy_off; else proxy_on; fi; }
proxy_status() { tspm status "$@"; }
proxy_ping() { tspm ping "$@"; }
proxy_diagnose() { tspm diagnose "$@"; }
proxy_benchmark() { tspm benchmark "$@"; }
proxy_best() { tspm best "$@"; }
proxy_switch() { tspm switch "$@"; }
proxy_dash() {
    local _tspm_rc
    tspm dash "$@"
    _tspm_rc=$?
    if [ -f "$HOME/.tspm-eval" ]; then
        if [ -f "$HOME/.tspm-debug-enabled" ]; then
            echo "[DEBUG-SHELL] proxy_dash: Found $HOME/.tspm-eval. Content:"
            cat "$HOME/.tspm-eval"
            echo
        fi
        eval "$(cat "$HOME/.tspm-eval")"
        rm -f "$HOME/.tspm-eval"
    else
        if [ -f "$HOME/.tspm-debug-enabled" ]; then
            echo "[DEBUG-SHELL] proxy_dash: $HOME/.tspm-eval not found."
        fi
    fi
    return $_tspm_rc
}
proxy_run() { tspm run -- "$@"; }
