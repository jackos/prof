use clap::Parser;
use color_eyre::Result;
use prof::{check_commands, heap, leak, Prof};

/// Tools to profile your Rust code
#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
enum Cargo {
    #[clap(subcommand)]
    Prof(Prof),
}

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .display_env_section(false)
        .install()?;
    let Cargo::Prof(prof) = Cargo::parse();
    match prof {
        Prof::Heap(x) => heap(x, Some(cargo_build)),
        Prof::Leak(x) => leak(x, Some(cargo_build)),
    }
}

pub fn cargo_build(bin: &Option<String>) -> Result<Option<String>> {
    check_commands(&["cargo"])?;
    cargo_prof::cargo_build(bin)?;
    Ok(Some(cargo_prof::get_bin()?))
}
