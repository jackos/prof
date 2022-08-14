use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

use clap::Parser;
use color_eyre::Result;
use prof::{cache, heap, leak, Commands, Prof};

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
    let prof = Prof::parse();

    match &prof.command {
        Commands::Heap(x) => heap(&prof, x, None),
        Commands::Leak(x) => leak(&prof, x, None),
        Commands::Cache(x) => cache(&prof, x, None),
    }
}
