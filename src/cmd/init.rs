use crate::cli::ShellType;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

/// Generates shell initialization code and completions for `zsh` or `bash`.
pub fn generate_shell_init<C: CommandFactory>(shell_type: &ShellType) {
    let mut cmd = C::command();
    match shell_type {
        ShellType::Zsh => {
            println!(
                r#"# proxy-cli Zsh Integration
proxy() {{
    if command -v proxy-cli >/dev/null 2>&1; then
        if [[ "$1" == "on" || "$1" == "off" ]]; then
            eval "$(proxy-cli env "$1")"
        elif [[ "$1" == "use" ]]; then
            proxy-cli profile use "$2"
            [ -n "$ALL_PROXY" ] && eval "$(proxy-cli env on)"
        elif [[ "$1" == "switch" ]]; then
            proxy-cli profile select
            [ -n "$ALL_PROXY" ] && eval "$(proxy-cli env on)"
        elif [[ "$1" == "best" ]]; then
            proxy-cli profile best
            [ -n "$ALL_PROXY" ] && eval "$(proxy-cli env on)"
        else
            proxy-cli "$@"
            if [ -f "$HOME/.proxy-cli-eval" ]; then
                echo "[DEBUG-SHELL] Found $HOME/.proxy-cli-eval. Content:"
                cat "$HOME/.proxy-cli-eval"
                echo
                eval "$(cat "$HOME/.proxy-cli-eval")"
                rm -f "$HOME/.proxy-cli-eval"
            else
                echo "[DEBUG-SHELL] $HOME/.proxy-cli-eval not found after proxy-cli exit."
            fi
        fi
    else
        echo "❌ proxy-cli binary not found in PATH"
    fi
}}

proxy_on() {{ eval "$(proxy-cli env on)"; }}
proxy_off() {{ eval "$(proxy-cli env off)"; }}
proxy_toggle() {{ if [ -n "$ALL_PROXY" ]; then proxy_off; else proxy_on; fi; }}
proxy_status() {{ proxy-cli status "$@"; }}
proxy_ping() {{ proxy-cli ping "$@"; }}
proxy_diagnose() {{ proxy-cli diagnose "$@"; }}
proxy_benchmark() {{ proxy-cli benchmark "$@"; }}
proxy_best() {{ proxy-cli best "$@"; }}
proxy_switch() {{ proxy-cli switch "$@"; }}
proxy_dash() {{ 
    proxy-cli dash "$@"
    if [ -f "$HOME/.proxy-cli-eval" ]; then
        echo "[DEBUG-SHELL] proxy_dash: Found $HOME/.proxy-cli-eval. Content:"
        cat "$HOME/.proxy-cli-eval"
        echo
        eval "$(cat "$HOME/.proxy-cli-eval")"
        rm -f "$HOME/.proxy-cli-eval"
    else
        echo "[DEBUG-SHELL] proxy_dash: $HOME/.proxy-cli-eval not found."
    fi
}}
proxy_run() {{ proxy-cli run -- "$@"; }}
prompt_proxy_status() {{ proxy-cli prompt; }}
"#
            );

            let mut buf = Vec::new();
            generate(Shell::Zsh, &mut cmd, "proxy-cli", &mut buf);
            if let Ok(compl_str) = String::from_utf8(buf) {
                println!("{}", compl_str);
                println!("compdef _proxy-cli proxy 2>/dev/null || true");
            }
        }
        ShellType::Bash => {
            println!(
                r#"# proxy-cli Bash Integration
proxy() {{
    if command -v proxy-cli >/dev/null 2>&1; then
        if [ "$1" = "on" ] || [ "$1" = "off" ]; then
            eval "$(proxy-cli env "$1")"
        elif [ "$1" = "use" ]; then
            proxy-cli profile use "$2"
            [ -n "$ALL_PROXY" ] && eval "$(proxy-cli env on)"
        elif [ "$1" = "switch" ]; then
            proxy-cli profile select
            [ -n "$ALL_PROXY" ] && eval "$(proxy-cli env on)"
        elif [ "$1" = "best" ]; then
            proxy-cli profile best
            [ -n "$ALL_PROXY" ] && eval "$(proxy-cli env on)"
        else
            proxy-cli "$@"
            if [ -f "$HOME/.proxy-cli-eval" ]; then
                echo "[DEBUG-SHELL] Found $HOME/.proxy-cli-eval. Content:"
                cat "$HOME/.proxy-cli-eval"
                echo
                eval "$(cat "$HOME/.proxy-cli-eval")"
                rm -f "$HOME/.proxy-cli-eval"
            else
                echo "[DEBUG-SHELL] $HOME/.proxy-cli-eval not found after proxy-cli exit."
            fi
        fi
    else
        echo "❌ proxy-cli binary not found in PATH"
    fi
}}

proxy_on() {{ eval "$(proxy-cli env on)"; }}
proxy_off() {{ eval "$(proxy-cli env off)"; }}
proxy_toggle() {{ if [ -n "$ALL_PROXY" ]; then proxy_off; else proxy_on; fi; }}
proxy_status() {{ proxy-cli status "$@"; }}
proxy_ping() {{ proxy-cli ping "$@"; }}
proxy_diagnose() {{ proxy-cli diagnose "$@"; }}
proxy_benchmark() {{ proxy-cli benchmark "$@"; }}
proxy_best() {{ proxy-cli best "$@"; }}
proxy_switch() {{ proxy-cli switch "$@"; }}
proxy_dash() {{ 
    proxy-cli dash "$@"
    if [ -f "$HOME/.proxy-cli-eval" ]; then
        echo "[DEBUG-SHELL] proxy_dash: Found $HOME/.proxy-cli-eval. Content:"
        cat "$HOME/.proxy-cli-eval"
        echo
        eval "$(cat "$HOME/.proxy-cli-eval")"
        rm -f "$HOME/.proxy-cli-eval"
    else
        echo "[DEBUG-SHELL] proxy_dash: $HOME/.proxy-cli-eval not found."
    fi
}}
proxy_run() {{ proxy-cli run -- "$@"; }}
"#
            );

            let mut buf = Vec::new();
            generate(Shell::Bash, &mut cmd, "proxy-cli", &mut buf);
            if let Ok(compl_str) = String::from_utf8(buf) {
                println!("{}", compl_str);
                println!("complete -F _proxy-cli proxy 2>/dev/null || true");
            }
        }
    }
}
