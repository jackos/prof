use clap::Parser;
use color_eyre::Result;
use prof::{check_commands, valgrind_bytes, Prof, Valgrind, ValgrindBytes};

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
    let Prof::Valgrind(val) = prof;
    match val {
        Valgrind::Bytes(bytes_args) => cargo_valgrind_bytes(bytes_args),
    }
}

pub fn cargo_valgrind_bytes(mut bytes_args: ValgrindBytes) -> Result<()> {
    check_commands(&["cargo"])?;
    cargo_prof::cargo_build(&bytes_args.bin)?;
    if bytes_args.bin.is_none() {
        bytes_args.bin = Some(cargo_prof::get_bin()?);
    }
    valgrind_bytes(bytes_args)
}
