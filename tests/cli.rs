//! End-to-end tests driving the real binary.
//!
//! Every test runs against a throwaway config and settings file supplied
//! through `TSPM_CONFIG` / `TSPM_SETTINGS`, so nothing here can read or modify
//! the developer's own `~/.config/terminal-session-proxy-manager`.
//!
//! Commands that reach the network (`status`, `ping`, `speedtest`, `monitor`,
//! `benchmark`) are deliberately not exercised here — they would be slow and
//! flaky. Their pure logic is covered by unit tests instead.

use std::fs;
use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

/// A binary invocation pinned to an isolated config directory.
struct Cli {
    _dir: TempDir,
    config_path: std::path::PathBuf,
    settings_path: std::path::PathBuf,
}

impl Cli {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("could not create a temp dir");
        let config_path = dir.path().join("config.json");
        let settings_path = dir.path().join("settings.json");

        Self {
            _dir: dir,
            config_path,
            settings_path,
        }
    }

    /// Seeds the config file with explicit contents.
    fn with_config(self, contents: &str) -> Self {
        fs::write(&self.config_path, contents).expect("could not seed the config");
        self
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("terminal-session-proxy-manager")
            .expect("the binary should be built");
        cmd.env("TSPM_CONFIG", &self.config_path)
            .env("TSPM_SETTINGS", &self.settings_path)
            // Keep assertions about output text stable regardless of the
            // developer's own locale preference.
            .env("TSPM_LANG", "en")
            .env("NO_COLOR", "1");
        cmd
    }

    fn config_contents(&self) -> String {
        fs::read_to_string(&self.config_path).expect("config should exist")
    }
}

const ONE_PROFILE: &str = r#"{
  "active_profile": "work",
  "profiles": {
    "work": { "name": "Work", "host": "127.0.0.1", "port": 1080, "protocol": "socks5" }
  }
}"#;

// ---------------------------------------------------------------- basics ---

#[test]
fn version_reports_the_crate_version() {
    Cli::new()
        .cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_never_leaks_a_raw_translation_key() {
    // `debug` shipped showing the literal `cmd_debug` because its translation
    // was missing; the key format is distinctive enough to assert on.
    for lang in ["en", "ru"] {
        let cli = Cli::new();
        let output = cli
            .cmd()
            .env("TSPM_LANG", lang)
            .arg("--help")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).unwrap();

        assert!(
            !stdout.contains("cmd_"),
            "{lang} --help contains an untranslated key:\n{stdout}"
        );
    }
}

#[test]
fn help_is_translated_into_each_language() {
    let cli = Cli::new();
    cli.cmd()
        .env("TSPM_LANG", "en")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Check network status"));

    cli.cmd()
        .env("TSPM_LANG", "ru")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Проверить"));
}

#[test]
fn an_unknown_subcommand_fails_rather_than_being_ignored() {
    Cli::new()
        .cmd()
        .arg("definitely-not-a-command")
        .assert()
        .failure();
}

// ------------------------------------------------------ config placement ---

#[test]
fn config_path_reports_the_overridden_location() {
    let cli = Cli::new();

    cli.cmd()
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(cli.config_path.to_str().unwrap()));
}

#[test]
fn a_first_run_creates_a_config_with_the_default_profiles() {
    let cli = Cli::new();

    cli.cmd().args(["profile", "list"]).assert().success();

    assert!(
        cli.config_path.exists(),
        "first run did not create a config"
    );
    assert!(cli.config_contents().contains("\"default\""));
}

#[test]
fn a_malformed_config_is_reported_and_left_byte_for_byte_intact() {
    // The single most destructive bug fixed in this release: `load()` used to
    // overwrite an unparsable config with defaults, discarding every profile.
    let broken = "{ \"profiles\": oops }";
    let cli = Cli::new().with_config(broken);

    cli.cmd()
        .args(["profile", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("is not valid JSON"));

    assert_eq!(cli.config_contents(), broken);
}

// ----------------------------------------------------------- env exports ---

#[test]
fn env_on_exports_every_managed_variable() {
    let cli = Cli::new().with_config(ONE_PROFILE);

    cli.cmd()
        .args(["env", "on"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "export http_proxy='http://127.0.0.1:1080';",
        ))
        .stdout(predicate::str::contains(
            "export ALL_PROXY='socks5://127.0.0.1:1080';",
        ))
        .stdout(predicate::str::contains("JAVA_TOOL_OPTIONS"));
}

#[test]
fn env_off_unsets_both_letter_cases() {
    Cli::new()
        .cmd()
        .args(["env", "off"])
        .assert()
        .success()
        .stdout(predicate::str::contains("unset "))
        .stdout(predicate::str::contains("http_proxy"))
        .stdout(predicate::str::contains("HTTP_PROXY"));
}

#[test]
fn the_generated_env_script_is_valid_shell() {
    // The whole feature is `eval "$(... env on)"`, so the output has to survive
    // a real shell rather than merely look right.
    let cli = Cli::new().with_config(ONE_PROFILE);
    let output = cli.cmd().args(["env", "on"]).output().unwrap();
    let script = String::from_utf8(output.stdout).unwrap();

    let evaluated = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("{script} printf %s \"$ALL_PROXY\""))
        .output()
        .unwrap();

    assert!(
        evaluated.status.success(),
        "generated script failed: {}",
        String::from_utf8_lossy(&evaluated.stderr)
    );
    assert!(
        String::from_utf8_lossy(&evaluated.stdout).ends_with("socks5://127.0.0.1:1080"),
        "ALL_PROXY was not set by the generated script"
    );
}

#[test]
fn export_renders_each_supported_format() {
    let cli = Cli::new().with_config(ONE_PROFILE);

    cli.cmd()
        .args(["export", "envfile"])
        .assert()
        .success()
        .stdout(predicate::str::contains("HTTP_PROXY=http://127.0.0.1:1080"));

    cli.cmd()
        .args(["export", "curl"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-x socks5://127.0.0.1:1080"));

    cli.cmd()
        .args(["export", "docker"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--build-arg http_proxy="));
}

#[test]
fn export_fails_loudly_when_no_profile_is_active() {
    // `proxy export envfile > .env` must not silently truncate the file.
    let cli = Cli::new().with_config(r#"{"active_profile":"missing","profiles":{}}"#);

    cli.cmd().args(["export", "envfile"]).assert().failure();
}

// -------------------------------------------------------------- profiles ---

#[test]
fn using_an_unknown_profile_exits_non_zero() {
    // `proxy profile use "$p" || fallback` depends on this; it used to print an
    // error and still exit 0.
    Cli::new()
        .with_config(ONE_PROFILE)
        .cmd()
        .args(["profile", "use", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nope"));
}

#[test]
fn removing_an_unknown_profile_exits_non_zero() {
    Cli::new()
        .with_config(ONE_PROFILE)
        .cmd()
        .args(["profile", "remove", "nope"])
        .assert()
        .failure();
}

#[test]
fn a_profile_can_be_added_then_listed_then_removed() {
    let cli = Cli::new().with_config(ONE_PROFILE);

    cli.cmd()
        .args([
            "profile", "set", "lab", "--name", "Lab", "--host", "10.0.0.5", "--port", "9050",
        ])
        .assert()
        .success();

    cli.cmd()
        .args(["profile", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lab"))
        .stdout(predicate::str::contains("10.0.0.5"));

    cli.cmd()
        .args(["profile", "remove", "lab"])
        .assert()
        .success();

    cli.cmd()
        .args(["profile", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lab").not());
}

#[test]
fn an_invalid_profile_is_rejected_before_it_reaches_the_config() {
    let cli = Cli::new().with_config(ONE_PROFILE);

    cli.cmd()
        .args(["profile", "set", "bad", "--protocol", "carrier-pigeon"])
        .assert()
        .failure();

    assert!(
        !cli.config_contents().contains("carrier-pigeon"),
        "an invalid profile was persisted anyway"
    );
}

#[test]
fn a_host_carrying_shell_syntax_is_rejected_at_the_boundary() {
    let cli = Cli::new().with_config(ONE_PROFILE);

    cli.cmd()
        .args([
            "profile",
            "set",
            "evil",
            "--host",
            "x;touch /tmp/tspm-pwned",
        ])
        .assert()
        .failure();

    assert!(!Path::new("/tmp/tspm-pwned").exists());
}

#[test]
fn using_a_profile_persists_the_choice() {
    let cli = Cli::new().with_config(ONE_PROFILE);

    cli.cmd()
        .args(["profile", "set", "second", "--host", "127.0.0.2"])
        .assert()
        .success();
    cli.cmd()
        .args(["profile", "use", "work"])
        .assert()
        .success();

    cli.cmd()
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"active_profile\": \"work\""));
}

// ------------------------------------------------------------- run/shell ---

#[test]
fn run_propagates_the_child_exit_code() {
    let cli = Cli::new().with_config(ONE_PROFILE);

    cli.cmd()
        .args(["run", "sh", "-c", "exit 7"])
        .assert()
        .code(7);
}

#[test]
fn run_injects_the_proxy_variables_into_the_child() {
    let cli = Cli::new().with_config(ONE_PROFILE);

    cli.cmd()
        .args(["run", "sh", "-c", "printf %s \"$ALL_PROXY\""])
        .assert()
        .success()
        .stdout("socks5://127.0.0.1:1080");
}

#[test]
fn run_forwards_flags_rather_than_parsing_them_itself() {
    // `run curl -sS ...` used to fail with "unexpected argument '-s'".
    let cli = Cli::new().with_config(ONE_PROFILE);

    cli.cmd()
        .args(["run", "printf", "%s-%s", "a", "b"])
        .assert()
        .success()
        .stdout("a-b");
}

#[test]
fn run_reports_a_missing_program_instead_of_pretending_it_worked() {
    Cli::new()
        .with_config(ONE_PROFILE)
        .cmd()
        .args(["run", "definitely-not-on-path-12345"])
        .assert()
        .failure();
}

#[test]
fn completions_are_generated_for_every_advertised_shell() {
    for shell in ["bash", "zsh", "fish", "powershell"] {
        Cli::new()
            .cmd()
            .args(["completions", shell])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}

#[test]
fn init_emits_a_sourceable_shell_function() {
    for shell in ["zsh", "bash"] {
        let cli = Cli::new();
        let output = cli.cmd().args(["init", shell]).output().unwrap();
        let script = String::from_utf8(output.stdout).unwrap();

        assert!(
            script.contains("proxy()"),
            "{shell} init defines no proxy()"
        );

        // The script is meant to be `eval`'d in the user's login shell, so a
        // syntax error would break every new terminal they open.
        let parsed = Command::new(if shell == "zsh" { "zsh" } else { "bash" })
            .args(["-n", "-c", &script])
            .output();
        if let Ok(parsed) = parsed {
            assert!(
                parsed.status.success(),
                "{shell} init script is not valid syntax: {}",
                String::from_utf8_lossy(&parsed.stderr)
            );
        }
    }
}

// -------------------------------------------------------------- settings ---

#[test]
fn the_language_can_be_changed_and_persists() {
    let cli = Cli::new();

    cli.cmd().args(["lang", "en"]).assert().success();

    let settings = fs::read_to_string(&cli.settings_path).unwrap();
    assert!(settings.contains("\"lang\": \"en\""));
}

#[test]
fn an_unsupported_language_is_rejected() {
    Cli::new()
        .cmd()
        .args(["lang", "klingon"])
        .assert()
        .failure();
}

#[test]
fn settings_path_reports_the_overridden_location() {
    let cli = Cli::new();

    cli.cmd()
        .args(["settings", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            cli.settings_path.to_str().unwrap(),
        ));
}

#[test]
fn the_config_flag_takes_precedence_over_the_environment() {
    let cli = Cli::new();
    let other = tempfile::tempdir().unwrap();
    let explicit = other.path().join("explicit.json");

    cli.cmd()
        .args([
            "--config-file",
            explicit.to_str().unwrap(),
            "config",
            "path",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(explicit.to_str().unwrap()));
}
