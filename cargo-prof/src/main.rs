use clap::Parser;
use color_eyre::Result;
use prof::{check_commands, heap, Heap, Prof};

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
        Prof::Heap(x) => cargo_valgrind_bytes(x),
    }
}

pub fn cargo_valgrind_bytes(mut bytes_args: Heap) -> Result<()> {
    check_commands(&["cargo"])?;
    cargo_prof::cargo_build(&bytes_args.bin)?;
    if bytes_args.bin.is_none() {
        bytes_args.bin = Some(cargo_prof::get_bin()?);
    }
    heap(bytes_args)
}
