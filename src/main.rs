use std::process::{Command, Stdio};

use cargo_metadata::{CargoOpt, MetadataCommand};
use clap::{Args, Parser, Subcommand};
use color_eyre::{
    eyre::{bail, Context},
    Help, Result,
};
use human_bytes::human_bytes;
use regex::bytes;

/// Tools to profile your Rust code
#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
enum Cargo {
    #[clap(subcommand)]
    Prof(Prof),
}

#[derive(Clone, Debug, Subcommand)]
#[clap(version)]
pub enum Prof {
    /// Valgrind is a memory leak detector and profiler
    #[clap(subcommand)]
    Valgrind(Valgrind),
}

#[derive(Clone, Debug, Subcommand)]
#[clap(version)]
pub enum Valgrind {
    /// Calculate the total bytes allocated to the heap by your program
    Bytes(Bytes),
}

#[derive(Args, Clone, Debug)]
#[clap(name = "new")]
pub struct Bytes {
    /// The binary target to profile
    #[clap(short, long)]
    bin: Option<String>,

    /// Bytes allocated by runtime, subtracted from the final result
    #[clap(short, long, default_value_t = 2157.0)]
    runtime_bytes: f64,

    /// Pass any additional args to the target binary with --
    #[clap(last = true)]
    target_args: Vec<String>,
}

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .display_env_section(false)
        .install()?;
    let Cargo::Prof(prof) = Cargo::parse();
    let Prof::Valgrind(val) = prof;
    match val {
        Valgrind::Bytes(bytes_args) => valgrind_bytes(bytes_args),
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
pub fn valgrind_bytes(bytes_args: Bytes) -> Result<()> {
    check_commands(&["valgrind", "cargo"])?;

    dbg!(&bytes_args.target_args);
    if cfg!(target_os = "windows") {
        bail!("Valgrind is not supported on Windows");
    }

    let mut command = std::process::Command::new("cargo");
    command.args(["build", "--release"]);

    if let Some(x) = &bytes_args.bin {
        command.args(["--bin", &x.clone()]);
    }

    let res = command.stderr(Stdio::inherit()).output()?;
    if !res.status.success() {
        bail!(
            "Cargo could not build the project: {}",
            String::from_utf8(res.stderr)?,
        )
    }

    let metadata = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .no_deps()
        .exec()?;

    let package_name = if let Some(x) = bytes_args.bin {
        x.clone()
    } else {
        let mut targets = Vec::new();
        for package in metadata.packages {
            targets.push(package.targets);
        }

        let targets: Vec<_> = targets
            .iter()
            .flatten()
            .filter(|x| !x.src_path.clone().into_string().contains(".cargo/registry"))
            .filter(|x| x.kind.contains(&"bin".to_string()))
            .collect();
        targets.get(0).expect("no target found").name.clone()
    };

    let mut command = std::process::Command::new("valgrind");

    command.args([format!("target/release/{package_name}")]);
    command.args(bytes_args.target_args);

    let res = String::from_utf8(command.output()?.stderr)?;

    let re = regex::Regex::new(r".*frees, ((\d|,)*).*allocated")?;
    let bytes = re
        .captures(&res)
        .expect("not found")
        .get(1)
        .expect("bytes not found in valgrind output");
    let mut bytes = bytes.as_str().replace(',', "").parse::<f64>()?;
    bytes = bytes - bytes_args.runtime_bytes;
    println!("total bytes allocated on heap: {}", human_bytes(bytes));

    Ok(())
}
