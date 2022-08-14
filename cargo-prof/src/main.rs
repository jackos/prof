use clap::Parser;
use color_eyre::Result;
use prof::utils::check_commands;
use prof::{cache, heap, leak, Commands, Prof};
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

use tracing_subscriber::filter::LevelFilter;

/// Tools to profile your Rust code
#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
enum Cargo {
    Prof(Prof),
}

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .display_env_section(false)
        .install()?;

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;
    let Cargo::Prof(prof) = Cargo::parse();

    match &prof.command {
        Commands::Heap(x) => heap(&prof, x, Some(cargo_build)),
        Commands::Leak(x) => leak(&prof, x, Some(cargo_build)),
        Commands::Cache(x) => cache(&prof, x, Some(cargo_build)),
    }
}

pub fn cargo_build(bin: &Option<String>) -> Result<Option<String>> {
    check_commands(&["cargo"])?;
    cargo_prof::cargo_build(bin)?;
    let res = cargo_prof::get_bin()?;
    Ok(Some(res))
}
