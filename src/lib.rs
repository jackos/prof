use clap::{Args, Subcommand};
use color_eyre::{
    eyre::{bail, Context},
    Help, Result,
};
use std::process::Command;
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
    Bytes(ValgrindBytes),
}

#[derive(Args, Clone, Debug)]
#[clap(name = "new")]
pub struct ValgrindBytes {
    /// The binary target to profile
    #[clap(short, long)]
    pub bin: Option<String>,

    /// Bytes allocated by runtime, subtracted from the final result
    #[clap(short, long, default_value_t = 2157.0)]
    pub runtime_bytes: f64,

    /// Pass any additional args to the target binary with --
    #[clap(last = true)]
    pub target_args: Vec<String>,
}

pub fn valgrind_bytes(bytes_args: ValgrindBytes) -> Result<()> {
    check_commands(&["valgrind"])?;

    dbg!(&bytes_args.target_args);
    if cfg!(target_os = "windows") {
        bail!("Valgrind is not supported on Windows");
    }
    let mut command = std::process::Command::new("valgrind");

    command.args([format!(
        "{}",
        bytes_args.bin.expect("no bin provided, give a --bin")
    )]);

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

    println!(
        "total bytes allocated on heap: {}",
        human_bytes::human_bytes(bytes)
    );

    Ok(())
}
