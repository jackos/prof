use std::process::Command;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

use clap::Parser;
use color_eyre::{eyre::Context, Help, Result};
use prof::{heap, leak, Prof};

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .display_env_section(false)
        .install()?;

    let subscriber = Registry::default()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;
    info!("it's working");
    let prof = Prof::parse();
    match prof {
        Prof::Heap(x) => heap(x, None),
        Prof::Leak(x) => leak(x, None),
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
