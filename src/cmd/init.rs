/// Generates shell initialization code for `zsh` or `bash`.
pub fn generate_shell_init(shell_type: &str) {
    match shell_type {
        "zsh" => {
            println!(r#"# proxy-cli Zsh Integration
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
proxy_run() {{ proxy-cli run -- "$@"; }}
prompt_proxy_status() {{ proxy-cli prompt; }}
"#);
        }
        "bash" => {
            println!(r#"# proxy-cli Bash Integration
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
proxy_run() {{ proxy-cli run -- "$@"; }}
"#);
        }
        _ => {
            eprintln!("Unsupported shell. Choose 'zsh' or 'bash'");
        }
    }
}
