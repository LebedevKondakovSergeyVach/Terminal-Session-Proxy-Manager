use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

/// Generates auto-completion scripts for the specified shell.
pub fn generate_completions<C: CommandFactory>(shell: Shell) -> Result<()> {
    let mut cmd = C::command();
    generate(
        shell,
        &mut cmd,
        "terminal-session-proxy-manager",
        &mut io::stdout(),
    );
    Ok(())
}
