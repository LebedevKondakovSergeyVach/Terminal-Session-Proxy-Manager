//! The hand-off files that let an ephemeral process change its parent shell.
//!
//! A child process cannot modify its parent's environment, so the TUI writes
//! the `export` lines it wants applied into a file in `$HOME`, and the shell
//! function installed by `init` evaluates and deletes that file when the binary
//! exits. Three modules used to hard-code these filenames independently; a
//! typo in any one of them silently broke the hand-off with no error anywhere.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

/// File the shell wrapper evaluates and removes after the binary exits.
const EVAL_FILE: &str = ".terminal-session-proxy-manager-eval";
/// Marker file whose presence turns on hand-off debug logging.
const DEBUG_FLAG_FILE: &str = ".terminal-session-proxy-manager-debug-enabled";
/// Append-only log written only while the debug marker exists.
const DEBUG_LOG_FILE: &str = ".terminal-session-proxy-manager-debug.log";

fn in_home(name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(name))
}

/// Path of the file the shell wrapper evaluates.
#[must_use]
pub fn eval_file() -> Option<PathBuf> {
    in_home(EVAL_FILE)
}

/// Path of the debug marker file.
#[must_use]
pub fn debug_flag_file() -> Option<PathBuf> {
    in_home(DEBUG_FLAG_FILE)
}

/// Path of the debug log file.
#[must_use]
pub fn debug_log_file() -> Option<PathBuf> {
    in_home(DEBUG_LOG_FILE)
}

/// Whether hand-off debug logging is currently enabled.
#[must_use]
pub fn is_debug_enabled() -> bool {
    debug_flag_file().is_some_and(|p| p.exists())
}

/// Turns hand-off debug logging on or off.
///
/// # Errors
/// Returns an error if the home directory is unknown or the marker file cannot
/// be created or removed.
pub fn set_debug(enabled: bool) -> Result<()> {
    let path = debug_flag_file().context("could not determine the home directory")?;

    if enabled {
        fs::write(&path, b"").with_context(|| format!("could not create {}", path.display()))?;
    } else if path.exists() {
        fs::remove_file(&path).with_context(|| format!("could not remove {}", path.display()))?;
    }
    Ok(())
}

/// Writes shell statements for the parent shell to evaluate on exit.
///
/// # Errors
/// Returns an error if the home directory is unknown or the file cannot be written.
pub fn write_exports(statements: &[String]) -> Result<()> {
    let path = eval_file().context("could not determine the home directory")?;
    let body = statements.join(" ");

    fs::write(&path, &body).with_context(|| format!("could not write {}", path.display()))?;
    log_debug(&format!("wrote {} with: {body}", path.display()));
    Ok(())
}

/// Appends a line to the debug log, but only when debugging is enabled.
///
/// Deliberately silent on failure: a diagnostic aid must never be the reason a
/// command fails.
pub fn log_debug(message: &str) {
    if !is_debug_enabled() {
        return;
    }

    let Some(path) = debug_log_file() else { return };
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        use std::io::Write;
        let _ = writeln!(file, "[tspm] {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_handoff_paths_sit_directly_in_the_home_directory() {
        let home = dirs::home_dir().unwrap();

        for path in [
            eval_file().unwrap(),
            debug_flag_file().unwrap(),
            debug_log_file().unwrap(),
        ] {
            assert_eq!(path.parent().unwrap(), home);
        }
    }

    #[test]
    fn the_three_handoff_files_have_distinct_names() {
        let names = [EVAL_FILE, DEBUG_FLAG_FILE, DEBUG_LOG_FILE];
        let unique: std::collections::BTreeSet<_> = names.iter().collect();

        assert_eq!(unique.len(), names.len());
    }

    #[test]
    fn logging_while_disabled_is_a_no_op_and_never_panics() {
        // The marker file is absent in CI, so this exercises the early return.
        if !is_debug_enabled() {
            log_debug("this must not create a log file");
            assert!(!debug_log_file().unwrap().exists());
        }
    }
}
