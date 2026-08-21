# ==========================================================
# TERMINAL SESSION PROXY MANAGER — BASH INTEGRATION
# ==========================================================
#
# Generated from: terminal-session-proxy-manager init bash
#
# Prefer the dynamic form in your rc file — it can never go stale:
#   eval "$(terminal-session-proxy-manager init bash)"
#
# This checked-in copy exists for setups that cannot run a command at
# shell startup. It omits the completion script that `init` also emits;
# get that with `terminal-session-proxy-manager completions bash`.

# terminal-session-proxy-manager Bash Integration
proxy() {
    if command -v terminal-session-proxy-manager >/dev/null 2>&1; then
        if [ "$1" = "on" ] || [ "$1" = "off" ]; then
            eval "$(terminal-session-proxy-manager env "$1")"
        elif [ "$1" = "use" ] || [ "$1" = "switch" ] || [ "$1" = "best" ]; then
            case "$1" in
                use)    terminal-session-proxy-manager profile use "$2" ;;
                switch) terminal-session-proxy-manager profile select ;;
                best)   terminal-session-proxy-manager profile best ;;
            esac
            # Capture first: the re-apply test used to become the function's
            # own status, so `proxy use work` reported failure on success when
            # the proxy was off, and success on failure when it was on.
            _tspm_rc=$?
            if [ $_tspm_rc -eq 0 ] && [ -n "$ALL_PROXY" ]; then
                eval "$(terminal-session-proxy-manager env on)"
            fi
            return $_tspm_rc
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
        echo "❌ terminal-session-proxy-manager binary not found in PATH"
    fi
}

proxy_on() { eval "$(terminal-session-proxy-manager env on)"; }
proxy_off() { eval "$(terminal-session-proxy-manager env off)"; }
proxy_toggle() { if [ -n "$ALL_PROXY" ]; then proxy_off; else proxy_on; fi; }
proxy_status() { terminal-session-proxy-manager status "$@"; }
proxy_ping() { terminal-session-proxy-manager ping "$@"; }
proxy_diagnose() { terminal-session-proxy-manager diagnose "$@"; }
proxy_benchmark() { terminal-session-proxy-manager benchmark "$@"; }
proxy_best() { terminal-session-proxy-manager best "$@"; }
proxy_switch() { terminal-session-proxy-manager switch "$@"; }
proxy_dash() { 
    terminal-session-proxy-manager dash "$@"
    if [ -f "$HOME/.terminal-session-proxy-manager-eval" ]; then
        if [ -f "$HOME/.terminal-session-proxy-manager-debug-enabled" ]; then
            echo "[DEBUG-SHELL] proxy_dash: Found $HOME/.terminal-session-proxy-manager-eval. Content:"
            cat "$HOME/.terminal-session-proxy-manager-eval"
            echo
        fi
        eval "$(cat "$HOME/.terminal-session-proxy-manager-eval")"
        rm -f "$HOME/.terminal-session-proxy-manager-eval"
    else
        if [ -f "$HOME/.terminal-session-proxy-manager-debug-enabled" ]; then
            echo "[DEBUG-SHELL] proxy_dash: $HOME/.terminal-session-proxy-manager-eval not found."
        fi
    fi
}
proxy_run() { terminal-session-proxy-manager run -- "$@"; }
