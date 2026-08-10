use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

pub fn generate_completions<C: CommandFactory>(shell: Shell) -> Result<()> {
    let mut cmd = C::command();
    generate(shell, &mut cmd, "proxy-cli", &mut io::stdout());
    Ok(())
}
