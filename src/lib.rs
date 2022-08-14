use clap::{AppSettings, Args, Parser};
use color_eyre::{
    eyre::{bail, eyre, Context},
    Help, Result,
};
use regex::{Captures, Match, SubCaptureMatches};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, process::Command};
use tracing::warn;
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

#[derive(Clone, Debug, Parser)]
#[clap(global_setting(AppSettings::DisableHelpFlag))]
#[clap(version)]
#[clap(name = "prof")]
#[clap(bin_name = "prof")]
pub enum Prof {
    /// Output the total bytes allocated and freed by the program
    Heap(Heap),
    /// Output leaked bytes from the program
    Leak(Leak),
}

#[derive(Args, Clone, Debug)]
#[clap(name = "new")]
pub struct Leak {
    /// The binary target to profile
    #[clap(short, long)]
    pub bin: Option<String>,

    /// Bytes allocate    /// Instead of human readable, show total bytes as a single int
    #[clap(short, long)]
    pub human_readable: bool,

    /// Pass any additional args to the target binary with --
    #[clap(last = true)]
    pub target_args: Vec<String>,
}

#[derive(Args, Clone, Debug)]
#[clap(name = "new")]
pub struct Heap {
    /// The binary target to profile
    #[clap(short, long)]
    pub bin: Option<String>,

    /// Bytes allocated by runtime, subtracted from the final result
    #[clap(short, long, default_value_t = 2157)]
    pub runtime_bytes: i64,

    /// Instead of human readable, show total bytes as a single int
    #[clap(short, long)]
    pub human_readable: bool,

    /// Pass any additional args to the target binary with --
    #[clap(last = true)]
    pub target_args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HeapSummary {
    pub allocated_total: u64,
    pub frees: u64,
    pub allocations: u64,
    pub allocated_at_exit: u64,
    pub blocks_at_exit: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HeapSummaryHuman {
    pub allocated_total: String,
    pub frees: u64,
    pub allocations: u64,
    pub allocated_at_exit: String,
    pub blocks_at_exit: u64,
}

struct LeakSummary {
    definitely: u64,
    indirectly: u64,
    possibly: u64,
    reachable: u64,
    supressed: u64,
    definitely_blocks: u64,
    indrectly_blocks: u64,
    possibly_blocks: u64,
    reachable_blocks: u64,
    supressed_blocks: u64,
}

pub fn valgrind(bin: Option<String>, target_args: Vec<String>) -> Result<String> {
    if cfg!(target_os = "windows") {
        bail!("Valgrind is not supported on Windows");
    }
    check_commands(&["valgrind"])?;

    let mut command = std::process::Command::new("valgrind");
    command.args([format!("{}", bin.expect("provide a --bin <BIN>"))]);

    command.args(target_args);

    Ok(String::from_utf8(command.output()?.stderr)?)
}

pub fn leak(
    mut args: Leak,
    cargo_fn: Option<fn(bin: &Option<String>) -> Result<Option<String>>>,
) -> Result<()> {
    if let Some(cargo_fn) = cargo_fn {
        args.bin = cargo_fn(&args.bin)?;
    };
    let res = valgrind(args.bin, args.target_args)?;
    let exit_cap = Capture::new(r".*in use at exit\D*([\d|,]*)\D*([\d|,]*)", &res)
        .context("Valgrind output")?;
    let mut exit = exit_cap.iter_next();

    let total_cap = Capture::new(
        r".*total heap usage: ([\d|,]*)\D*([\d|,]*)\D*([\d|,]*)",
        &res,
    )
    .context("Valgrind output")?;
    let mut total = total_cap.iter_next();

    let heap_usage = HeapSummary {
        allocated_at_exit: parse_valgrind("in use at exit", exit.next()),
        blocks_at_exit: parse_valgrind("in use at exit blocks", exit.next()),
        allocations: parse_valgrind("heap allocated", total.next()),
        frees: parse_valgrind("heap frees", total.next()),
        allocated_total: parse_valgrind("total heap usage", total.next()),
    };

    if args.human_readable {
        let human_readble = HeapSummaryHuman {
            allocated_at_exit: human_bytes(heap_usage.allocated_at_exit),
            blocks_at_exit: heap_usage.blocks_at_exit,
            allocations: heap_usage.allocations,
            frees: heap_usage.frees,
            allocated_total: human_bytes(heap_usage.allocated_total),
        };
        let parsed = serde_yaml::to_string(&human_readble)?;
        println!("{parsed}");
    } else {
        let parsed = serde_json::to_string(&heap_usage)?;
        println!("{parsed}");
    }

    Ok(())
}

pub fn heap(
    mut args: Heap,
    cargo_fn: Option<fn(bin: &Option<String>) -> Result<Option<String>>>,
) -> Result<()> {
    if let Some(cargo_fn) = cargo_fn {
        args.bin = cargo_fn(&args.bin)?;
    };
    let res = valgrind(args.bin, args.target_args)?;
    let exit_cap = Capture::new(r".*in use at exit\D*([\d|,]*)\D*([\d|,]*)", &res)
        .context("Valgrind output")?;
    let mut exit = exit_cap.iter_next();

    let total_cap = Capture::new(
        r".*total heap usage: ([\d|,]*)\D*([\d|,]*)\D*([\d|,]*)",
        &res,
    )
    .context("Valgrind output")?;
    let mut total = total_cap.iter_next();

    let heap_usage = HeapSummary {
        allocated_at_exit: parse_valgrind("in use at exit", exit.next()),
        blocks_at_exit: parse_valgrind("in use at exit blocks", exit.next()),
        allocations: parse_valgrind("heap allocated", total.next()),
        frees: parse_valgrind("heap frees", total.next()),
        allocated_total: parse_valgrind("total heap usage", total.next()),
    };

    if args.human_readable {
        let human_readble = HeapSummaryHuman {
            allocated_at_exit: human_bytes(heap_usage.allocated_at_exit),
            blocks_at_exit: heap_usage.blocks_at_exit,
            allocations: heap_usage.allocations,
            frees: heap_usage.frees,
            allocated_total: human_bytes(heap_usage.allocated_total),
        };
        let parsed = serde_yaml::to_string(&human_readble)?;
        println!("{parsed}");
    } else {
        let parsed = serde_json::to_string(&heap_usage)?;
        println!("{parsed}");
    }

    Ok(())
}

/// Wrap a standard capture with a custom `new()` implmentation to de-depulicate code
struct Capture<'a>(Captures<'a>);

impl<'a> Capture<'a> {
    fn new(re: &str, output: &'a str) -> Result<Self> {
        let re = regex::Regex::new(re).expect("failed to compile regex");
        match re.captures(output) {
            Some(x) => Ok(Capture(x)),
            None => Err(eyre!("No match found for regex: {}", re)),
        }
    }
    /// Create an iterator over the captures but skip the fist one
    fn iter_next(&'a self) -> SubCaptureMatches<'a, 'a> {
        let mut x = self.iter();
        x.next();
        x
    }
}

impl<'a> Deref for Capture<'a> {
    type Target = Captures<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// pub fn regex_to_iter<'a>(re: &str, output: &'a str) -> Result<Capture<'a>> {
//     let re = regex::Regex::new(re)?;
//     let captures = re
//         .captures(output)
//         .expect("could not find heap usage in valgrind output");
//     // total
//     //     .next()
//     //     .ok_or_else(|| warn!("no line found for `total heap usage` in valgrind output"))
//     //     .unwrap();
//     Ok(Capture {
//         captures: re.captures(output).expect("could not find heap usage"),
//         iter: captures.iter(),
//     })
// }

pub fn warn_and_return(param_name: &str) -> u64 {
    warn!("{} not found in valgrind output", param_name);
    0
}

pub fn parse_valgrind(param_name: &str, param: Option<Option<Match>>) -> u64 {
    let re_match = match param {
        Some(x) => match x {
            Some(x) => x,
            None => return warn_and_return(param_name),
        },
        None => return warn_and_return(param_name),
    };

    let res = re_match.as_str().replace(',', "").parse::<u64>();
    match res {
        Ok(x) => x,
        Err(e) => {
            warn!("failed to parse int for param: {param_name}: {e}");
            0
        }
    }
}

/// Converts bytes to human-readable values
pub fn human_bytes(size: u64) -> String {
    let mut bytes = String::new();

    let mut kb = size / 1024;
    let b = size % 1024;

    let mut mb = kb / 1024;
    if mb > 0 {
        kb = kb % 1024;
    }
    let gb = mb / 1024;
    if gb > 0 {
        mb = mb % 1024;
        bytes.push_str(&format!("{gb}GB "))
    }
    if mb > 0 {
        bytes.push_str(&format!("{mb}MB "))
    }
    if kb > 0 {
        bytes.push_str(&format!("{kb}KB "))
    }
    if b > 0 {
        bytes.push_str(&format!("{b}B"))
    }
    bytes
}
