//! Terminal Session Proxy Manager — binary entry point.
//!
//! Deliberately thin: it resolves configuration, localises the help text, and
//! dispatches to [`terminal_session_proxy_manager::cmd`]. Logic lives in the
//! library so it can be tested without spawning a process.

use std::process::ExitCode;

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use colored::Colorize;
use terminal_session_proxy_manager::cli::{
    Cli, Commands, ConfigCommands, EnvMode, ProfileCommands, SettingsCommands,
};
use terminal_session_proxy_manager::cmd::{
    completions, dash, diagnose, env as env_cmd, export_cmd, git_cmd, import_cmd, init, monitor,
    ping, profile, settings, speedtest, status,
};
use terminal_session_proxy_manager::config::{
    AppConfig, AppSettings, I18n, set_config_path_override, set_settings_path_override,
};
use terminal_session_proxy_manager::{proxy_env, shell_handoff};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(code) => code,
        Err(err) => {
            // `{err:?}` renders the whole anyhow context chain, not just the
            // outermost message.
            eprintln!("{} {err:?}", "Error:".red().bold());
            ExitCode::FAILURE
        }
    }
}

/// Reads a `--flag value` or `--flag=value` pair straight out of argv.
///
/// The language and the config locations are needed *before* clap parses,
/// because the help text is localised as the command tree is built. clap
/// remains the authority for every value once parsing happens; this only
/// bootstraps those three settings.
fn preparse<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    // `run` is a trailing_var_arg subcommand, so everything after it belongs to
    // the child. Scanning past it let `proxy run mytool --lang en` retarget the
    // manager itself, changing which proxy the child was given.
    let scannable = args
        .iter()
        .position(|a| a == "run" || a == "--")
        .map_or(args.len(), |i| i);

    let mut iter = args[..scannable].iter();
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next().map(String::as_str);
        }
        if let Some(value) = arg.strip_prefix(&format!("{flag}=")) {
            return Some(value);
        }
    }
    None
}

async fn run() -> Result<ExitCode> {
    // NO_COLOR is honoured for any non-empty value, per the no-color.org spec.
    if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        colored::control::set_override(false);
    }

    let raw_args: Vec<String> = std::env::args().collect();
    if let Some(path) = preparse(&raw_args, "--settings-file") {
        set_settings_path_override(path.into());
    }
    if let Some(path) = preparse(&raw_args, "--config-file") {
        set_config_path_override(path.into());
    }

    let settings = AppSettings::load();
    let lang = preparse(&raw_args, "--lang")
        .map(str::to_string)
        .or_else(|| std::env::var("TSPM_LANG").ok())
        .unwrap_or(settings.lang);
    let i18n = I18n::load(&lang);

    let cli = Cli::from_arg_matches(&localised_command(&i18n).get_matches())?;
    let mut config = AppConfig::load();

    dispatch(cli, &mut config, &i18n).await
}

/// Builds the command tree with subcommand descriptions in the active language.
///
/// clap's derive attributes are static, so the bilingual doc comments in
/// `cli.rs` are replaced here with a single-language description at runtime.
fn localised_command(i18n: &I18n) -> clap::Command {
    let mut cmd = Cli::command();
    cmd.build(); // Build early to generate the 'help' subcommand

    cmd = cmd.about(i18n.t("cmd_about").to_string());

    for name in subcommand_names(&cmd) {
        let about = i18n.t(&format!("cmd_{name}")).to_string();
        cmd = cmd.mut_subcommand(name, |s| s.about(about));
    }
    
    // Explicitly localise the built-in 'help' command
    cmd = cmd.mut_subcommand("help", |s| s.about(i18n.t("cmd_help").to_string()));
    
    cmd
}

/// Names of every subcommand except clap's built-in `help`.
///
/// Derived from the command tree rather than a hand-maintained list, so a new
/// subcommand is localised automatically instead of printing a raw key.
fn subcommand_names(cmd: &clap::Command) -> Vec<String> {
    cmd.get_subcommands()
        .map(|s| s.get_name().to_string())
        .filter(|n| n != "help")
        .collect()
}

#[allow(clippy::too_many_lines)] // A flat dispatch table reads better than nested helpers.
async fn dispatch(cli: Cli, config: &mut AppConfig, i18n: &I18n) -> Result<ExitCode> {
    match cli.command {
        Commands::Status { json } => status::print_status(config, i18n, json).await?,
        Commands::Env { mode } => env_cmd::print_env_commands(&mode, config, i18n)?,
        Commands::Debug { mode } => {
            let enabled = matches!(mode, EnvMode::On);
            shell_handoff::set_debug(enabled)?;
            let key = if enabled {
                "debug_enabled"
            } else {
                "debug_disabled"
            };
            println!("{} {}", "✓".green(), i18n.t(key));
        }
        Commands::Git { mode } => git_cmd::handle_git_proxy(&mode, config, i18n)?,
        Commands::Export { format } => export_cmd::export_config(&format, config, i18n)?,
        Commands::Speedtest => speedtest::run_speedtest(config, i18n).await?,
        Commands::Monitor => monitor::run_monitor(config, i18n).await?,
        Commands::Lang { code } => settings::set_lang(code)?,
        Commands::Switch => profile::select_profile_interactive(config, i18n)?,
        Commands::Dash => dash::run_dashboard(config, i18n).await?,
        Commands::Benchmark => profile::run_benchmark(config, i18n).await?,
        Commands::Best => profile::select_best_profile(config, i18n).await?,
        Commands::Import { source } => import_cmd::import_profiles(config, i18n, &source).await?,
        Commands::Ping { timeout } => ping::run_ping(config, i18n, timeout).await?,
        Commands::Diagnose => diagnose::run_diagnose(config, i18n).await?,
        Commands::Prompt => print_prompt_segment(config),
        Commands::Run { cmd } => return run_through_proxy(&cmd, config, i18n),
        Commands::Init { shell } => init::generate_shell_init::<Cli>(&shell),
        Commands::Completions { shell } => completions::generate_completions::<Cli>(shell)?,

        Commands::Profile(profile_cmd) => match profile_cmd {
            ProfileCommands::List => profile::list_profiles(config, i18n),
            ProfileCommands::Select => profile::select_profile_interactive(config, i18n)?,
            ProfileCommands::Use { key } => profile::use_profile(config, i18n, &key)?,
            ProfileCommands::Set {
                key,
                name,
                port,
                host,
                protocol,
            } => profile::set_profile(config, i18n, key, name, port, host, protocol)?,
            ProfileCommands::Import { source } => {
                import_cmd::import_profiles(config, i18n, &source).await?;
            }
            ProfileCommands::Remove { key } => profile::remove_profile(config, i18n, &key)?,
            ProfileCommands::Benchmark => profile::run_benchmark(config, i18n).await?,
            ProfileCommands::Best => profile::select_best_profile(config, i18n).await?,
        },

        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Path => {
                println!("{}", i18n.t("config_active_path"));
                println!("{}", AppConfig::get_config_path().display());
            }
            ConfigCommands::Show => println!("{}", serde_json::to_string_pretty(config)?),
        },

        Commands::Settings(settings_cmd) => match settings_cmd {
            SettingsCommands::Path => settings::show_settings_path(i18n),
            SettingsCommands::Show => settings::show_settings()?,
            SettingsCommands::Set { config_path, lang } => {
                if let Some(path) = config_path {
                    settings::set_config_path(path, i18n)?;
                }
                if let Some(code) = lang {
                    settings::set_lang(code)?;
                }
            }
        },
    }

    Ok(ExitCode::SUCCESS)
}

/// Prints the prompt indicator, but only when the shell actually has a proxy set.
fn print_prompt_segment(config: &AppConfig) {
    let proxy_active = ["ALL_PROXY", "all_proxy", "http_proxy", "HTTP_PROXY"]
        .iter()
        .any(|name| std::env::var_os(name).is_some_and(|v| !v.is_empty()));

    if !proxy_active {
        return;
    }

    if let Some(p) = config.active_profile() {
        println!("\x1b[1;33m🌐 [{}:{}]\x1b[0m", p.name, p.port);
    }
}

/// Runs a command with the proxy environment applied, without touching the shell.
///
/// Propagates the child's exit status so `proxy run some-check && deploy`
/// behaves the same as running `some-check` directly.
fn run_through_proxy(cmd: &[String], config: &AppConfig, i18n: &I18n) -> Result<ExitCode> {
    let Some(vars) = proxy_env::proxy_env_vars(config) else {
        eprintln!("{}", i18n.t("proxy_load_failed").red().bold());
        return Ok(ExitCode::FAILURE);
    };

    let Some((program, args)) = cmd.split_first() else {
        // clap enforces `num_args = 1..`, so this is unreachable in practice.
        eprintln!("{}", i18n.t("run_needs_command").red().bold());
        return Ok(ExitCode::FAILURE);
    };

    let status = std::process::Command::new(program)
        .args(args)
        .envs(vars)
        .status()
        .map_err(|e| anyhow::anyhow!("could not run '{program}': {e}"))?;

    // A process killed by a signal has no exit code; 1 is the conventional
    // stand-in, since ExitCode cannot represent a signal.
    Ok(ExitCode::from(
        u8::try_from(status.code().unwrap_or(1)).unwrap_or(1),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal_session_proxy_manager::cli::LangCode;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn preparse_reads_the_space_separated_form() {
        let argv = args(&["tspm", "--lang", "en", "status"]);

        assert_eq!(preparse(&argv, "--lang"), Some("en"));
    }

    #[test]
    fn preparse_reads_the_equals_form() {
        let argv = args(&["tspm", "--lang=en", "status"]);

        assert_eq!(preparse(&argv, "--lang"), Some("en"));
    }

    #[test]
    fn preparse_finds_a_flag_placed_after_the_subcommand() {
        let argv = args(&["tspm", "status", "--config-file", "/tmp/c.json"]);

        assert_eq!(preparse(&argv, "--config-file"), Some("/tmp/c.json"));
    }

    #[test]
    fn preparse_returns_none_for_an_absent_flag() {
        assert_eq!(preparse(&args(&["tspm", "status"]), "--lang"), None);
    }

    #[test]
    fn preparse_does_not_match_a_longer_flag_sharing_the_prefix() {
        let argv = args(&["tspm", "--lang-extra", "x"]);

        assert_eq!(preparse(&argv, "--lang"), None);
    }

    #[test]
    fn preparse_tolerates_a_trailing_flag_with_no_value() {
        assert_eq!(preparse(&args(&["tspm", "--lang"]), "--lang"), None);
    }

    #[test]
    fn every_subcommand_has_a_localised_description_in_both_languages() {
        // The `debug` subcommand once printed the literal key `cmd_debug` in
        // `--help` because its translation was never added.
        let cmd = Cli::command();

        for lang in ["ru", "en"] {
            let i18n = I18n::load(lang);
            for name in subcommand_names(&cmd) {
                let key = format!("cmd_{name}");
                assert_ne!(
                    i18n.t(&key),
                    key,
                    "{lang}.json is missing '{key}', so --help would show the raw key"
                );
            }
        }
    }

    #[test]
    fn the_localised_command_tree_still_builds_and_validates() {
        for lang in ["ru", "en"] {
            localised_command(&I18n::load(lang)).debug_assert();
        }
    }

    #[test]
    fn localising_replaces_the_bilingual_derive_text() {
        let about = localised_command(&I18n::load("en"))
            .get_about()
            .unwrap()
            .to_string();

        assert_eq!(about, I18n::load("en").t("cmd_about"));
    }

    #[test]
    fn help_is_never_treated_as_a_localisable_subcommand() {
        assert!(!subcommand_names(&Cli::command()).contains(&"help".to_string()));
    }

    #[test]
    fn lang_codes_survive_the_round_trip_through_settings() {
        for code in [LangCode::Ru, LangCode::En] {
            assert_eq!(I18n::load(code.as_str()).lang(), code.as_str());
        }
    }
}
