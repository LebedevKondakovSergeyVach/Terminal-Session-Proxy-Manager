use crate::cli::ShellType;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

/// Generates shell initialization code and completions for `zsh` or `bash`.
pub fn generate_shell_init<C: CommandFactory>(shell_type: &ShellType) {
    let mut cmd = C::command();
    match shell_type {
        ShellType::Zsh => {
            println!(
                r#"# terminal-session-proxy-manager Zsh Integration
proxy() {{
    if command -v terminal-session-proxy-manager >/dev/null 2>&1; then
        if [[ "$1" == "on" || "$1" == "off" ]]; then
            eval "$(terminal-session-proxy-manager env "$1")"
        elif [[ "$1" == "use" ]]; then
            terminal-session-proxy-manager profile use "$2"
            [ -n "$ALL_PROXY" ] && eval "$(terminal-session-proxy-manager env on)"
        elif [[ "$1" == "switch" ]]; then
            terminal-session-proxy-manager profile select
            [ -n "$ALL_PROXY" ] && eval "$(terminal-session-proxy-manager env on)"
        elif [[ "$1" == "best" ]]; then
            terminal-session-proxy-manager profile best
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
        echo "❌ terminal-session-proxy-manager binary not found in PATH"
    fi
}}

proxy_on() {{ eval "$(terminal-session-proxy-manager env on)"; }}
proxy_off() {{ eval "$(terminal-session-proxy-manager env off)"; }}
proxy_toggle() {{ if [ -n "$ALL_PROXY" ]; then proxy_off; else proxy_on; fi; }}
proxy_status() {{ terminal-session-proxy-manager status "$@"; }}
proxy_ping() {{ terminal-session-proxy-manager ping "$@"; }}
proxy_diagnose() {{ terminal-session-proxy-manager diagnose "$@"; }}
proxy_benchmark() {{ terminal-session-proxy-manager benchmark "$@"; }}
proxy_best() {{ terminal-session-proxy-manager best "$@"; }}
proxy_switch() {{ terminal-session-proxy-manager switch "$@"; }}
proxy_dash() {{ 
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
}}
proxy_run() {{ terminal-session-proxy-manager run -- "$@"; }}
prompt_proxy_status() {{ terminal-session-proxy-manager prompt; }}
"#
            );

            let mut buf = Vec::new();
            generate(
                Shell::Zsh,
                &mut cmd,
                "terminal-session-proxy-manager",
                &mut buf,
            );
            if let Ok(compl_str) = String::from_utf8(buf) {
                println!("{}", compl_str);
                println!("compdef _terminal-session-proxy-manager proxy 2>/dev/null || true");
            }
        }
        ShellType::Bash => {
            println!(
                r#"# terminal-session-proxy-manager Bash Integration
proxy() {{
    if command -v terminal-session-proxy-manager >/dev/null 2>&1; then
        if [ "$1" = "on" ] || [ "$1" = "off" ]; then
            eval "$(terminal-session-proxy-manager env "$1")"
        elif [ "$1" = "use" ]; then
            terminal-session-proxy-manager profile use "$2"
            [ -n "$ALL_PROXY" ] && eval "$(terminal-session-proxy-manager env on)"
        elif [ "$1" = "switch" ]; then
            terminal-session-proxy-manager profile select
            [ -n "$ALL_PROXY" ] && eval "$(terminal-session-proxy-manager env on)"
        elif [ "$1" = "best" ]; then
            terminal-session-proxy-manager profile best
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
        echo "❌ terminal-session-proxy-manager binary not found in PATH"
    fi
}}

proxy_on() {{ eval "$(terminal-session-proxy-manager env on)"; }}
proxy_off() {{ eval "$(terminal-session-proxy-manager env off)"; }}
proxy_toggle() {{ if [ -n "$ALL_PROXY" ]; then proxy_off; else proxy_on; fi; }}
proxy_status() {{ terminal-session-proxy-manager status "$@"; }}
proxy_ping() {{ terminal-session-proxy-manager ping "$@"; }}
proxy_diagnose() {{ terminal-session-proxy-manager diagnose "$@"; }}
proxy_benchmark() {{ terminal-session-proxy-manager benchmark "$@"; }}
proxy_best() {{ terminal-session-proxy-manager best "$@"; }}
proxy_switch() {{ terminal-session-proxy-manager switch "$@"; }}
proxy_dash() {{ 
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
}}
proxy_run() {{ terminal-session-proxy-manager run -- "$@"; }}
"#
            );

            let mut buf = Vec::new();
            generate(
                Shell::Bash,
                &mut cmd,
                "terminal-session-proxy-manager",
                &mut buf,
            );
            if let Ok(compl_str) = String::from_utf8(buf) {
                println!("{}", compl_str);
                println!("complete -F _terminal-session-proxy-manager proxy 2>/dev/null || true");
            }
        }
    }
}
