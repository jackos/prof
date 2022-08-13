use std::process::Command;

use clap::{Parser, Subcommand};
use color_eyre::{eyre::Context, Help, Result};
use prof::ValgrindBytes;

#[derive(Clone, Debug, Parser)]
#[clap(version)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
pub enum Prof {
    /// Valgrind is a memory leak detector and profiler
    #[clap(subcommand)]
    Valgrind(Valgrind),
}

#[derive(Clone, Debug, Subcommand)]
#[clap(version)]
pub enum Valgrind {
    /// Calculate the total bytes allocated to the heap by your program
    Bytes(ValgrindBytes),
}

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .display_env_section(false)
        .install()?;
    let prof = Prof::parse();
    let Prof::Valgrind(val) = prof;
    match val {
        Valgrind::Bytes(bytes_args) => prof::valgrind_bytes(bytes_args),
    }
}

pub fn check_commands(commands: &[&str]) -> Result<()> {
    for command in commands {
        Command::new(command)
            .output()
            .context(format!("Command: {command} not found"))
            .with_suggestion(|| {
                format!("make sure {command} is installed and it's on your path: https://command-not-found.com/{command}")
            })?;
    }
    Ok(())
}
