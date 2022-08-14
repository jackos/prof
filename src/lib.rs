use clap::{Args, Parser, Subcommand};
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

#[derive(Parser, Debug)]
#[clap(version)]
#[clap(name = "prof")]
#[clap(bin_name = "prof")]
pub struct Prof {
    #[clap(subcommand)]
    pub command: Commands,

    /// The binary target to profile
    #[clap(short, long, global = true)]
    pub bin: Option<String>,

    /// JSON output with total bytes. Deafults to YAML with human readable bytes
    #[clap(short, long, global = true)]
    pub json: bool,

    /// Pass any additional args to the target binary with --
    #[clap(last = true, global = true)]
    pub target_args: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Output the total bytes allocated and freed by the program
    Heap(Heap),
    /// Output leaked bytes from the program
    Leak(Leak),
}

#[derive(Args, Clone, Debug)]
#[clap(name = "new")]
pub struct Leak {}

#[derive(Args, Clone, Debug)]
#[clap(name = "new")]
pub struct Heap {
    /// Subtract bytes from total allocated
    #[clap(short, long, default_value_t = 0)]
    pub subtract_bytes: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HeapSummary {
    pub allocated_total: i64,
    pub frees: i64,
    pub allocations: i64,
    pub allocated_at_exit: i64,
    pub blocks_at_exit: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HeapSummaryHuman {
    pub allocated_total: String,
    pub frees: i64,
    pub allocations: i64,
    pub allocated_at_exit: String,
    pub blocks_at_exit: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LeakSummary {
    pub definitely_lost: i64,
    pub indirectly_lost: i64,
    pub possibly_lost: i64,
    pub still_reachable: i64,
    pub supressed: i64,
    pub definitely_lost_blocks: i64,
    pub indrectly_lost_blocks: i64,
    pub possibly_lost_blocks: i64,
    pub still_reachable_blocks: i64,
    pub supressed_blocks: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LeakSummaryHuman {
    pub definitely_lost: String,
    pub indirectly_lost: String,
    pub possibly_lost: String,
    pub still_reachable: String,
    pub supressed: String,
    pub definitely_lost_blocks: i64,
    pub indrectly_lost_blocks: i64,
    pub possibly_lost_blocks: i64,
    pub still_reachable_blocks: i64,
    pub supressed_blocks: i64,
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

pub fn heap(
    args: &Prof,
    heap_args: &Heap,
    cargo_fn: Option<fn(bin: &Option<String>) -> Result<Option<String>>>,
) -> Result<()> {
    let mut bin = args.bin.clone();
    if let Some(cargo_fn) = cargo_fn {
        bin = cargo_fn(&args.bin)?;
    };
    let res = valgrind(bin, args.target_args.clone())?;
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
        allocated_total: parse_valgrind("total heap usage", total.next())
            - heap_args.subtract_bytes,
    };

    if args.json {
        let parsed = serde_json::to_string(&heap_usage)?;
        println!("{parsed}");
    } else {
        let human_readble = HeapSummaryHuman {
            allocated_at_exit: human_bytes(heap_usage.allocated_at_exit),
            blocks_at_exit: heap_usage.blocks_at_exit,
            allocations: heap_usage.allocations,
            frees: heap_usage.frees,
            allocated_total: human_bytes(heap_usage.allocated_total),
        };
        let parsed = serde_yaml::to_string(&human_readble)?;
        println!("{parsed}");
    }

    Ok(())
}

pub fn leak(
    args: &Prof,
    _leak_args: &Leak,
    cargo_fn: Option<fn(bin: &Option<String>) -> Result<Option<String>>>,
) -> Result<()> {
    let mut bin = args.bin.clone();
    if let Some(cargo_fn) = cargo_fn {
        bin = cargo_fn(&args.bin)?;
    };
    let res = valgrind(bin, args.target_args.clone())?;

    let definite_cap = Capture::new(r".*definitely lost: ([\d|,]*)\D*([\d|,]*)", &res)
        .context("Valgrind output")?;
    let mut definite = definite_cap.iter_next();

    let indirect_cap = Capture::new(r".*indirectly lost: ([\d|,]*)\D*([\d|,]*)", &res)
        .context("Valgrind output")?;
    let mut indirect = indirect_cap.iter_next();

    let possible_cap =
        Capture::new(r".*possibly lost: ([\d|,]*)\D*([\d|,]*)", &res).context("Valgrind output")?;
    let mut possible = possible_cap.iter_next();

    let reachable_cap = Capture::new(r".*still reachable: ([\d|,]*)\D*([\d|,]*)", &res)
        .context("Valgrind output")?;
    let mut reachable = reachable_cap.iter_next();

    let suppressed_cap =
        Capture::new(r".*suppressed: ([\d|,]*)\D*([\d|,]*)", &res).context("Valgrind output")?;
    let mut suppressed = suppressed_cap.iter_next();

    let leak_summary = LeakSummary {
        definitely_lost: parse_valgrind("definitely_lost", definite.next()),
        definitely_lost_blocks: parse_valgrind("definitely_lost_blocks", definite.next()),

        indirectly_lost: parse_valgrind("indirectly_lost", indirect.next()),
        indrectly_lost_blocks: parse_valgrind("indirect_lost_blocks", indirect.next()),

        possibly_lost: parse_valgrind("possibly_lost", possible.next()),
        possibly_lost_blocks: parse_valgrind("possibly_lost_blocks", possible.next()),

        still_reachable: parse_valgrind("still_reachable", reachable.next()),
        still_reachable_blocks: parse_valgrind("still_reachable_blocks", reachable.next()),

        supressed: parse_valgrind("supressed", suppressed.next()),
        supressed_blocks: parse_valgrind("supressed_blocks", suppressed.next()),
    };

    if !args.json {
        let human_readble = LeakSummaryHuman {
            definitely_lost: human_bytes(leak_summary.definitely_lost),
            definitely_lost_blocks: leak_summary.definitely_lost,
            indirectly_lost: human_bytes(leak_summary.indirectly_lost),
            indrectly_lost_blocks: leak_summary.indrectly_lost_blocks,
            possibly_lost: human_bytes(leak_summary.possibly_lost),
            possibly_lost_blocks: leak_summary.possibly_lost_blocks,
            still_reachable: human_bytes(leak_summary.still_reachable),
            still_reachable_blocks: leak_summary.still_reachable_blocks,
            supressed: human_bytes(leak_summary.supressed),
            supressed_blocks: leak_summary.supressed_blocks,
        };
        let parsed = serde_yaml::to_string(&human_readble)?;
        println!("{parsed}");
    } else {
        let parsed = serde_json::to_string(&leak_summary)?;
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

pub fn warn_and_return(param_name: &str) -> i64 {
    warn!("{} not found in valgrind output", param_name);
    0
}

pub fn parse_valgrind(param_name: &str, param: Option<Option<Match>>) -> i64 {
    let re_match = match param {
        Some(x) => match x {
            Some(x) => x,
            None => return warn_and_return(param_name),
        },
        None => return warn_and_return(param_name),
    };

    let res = re_match.as_str().replace(',', "").parse::<i64>();
    match res {
        Ok(x) => x,
        Err(e) => {
            warn!("failed to parse int for param: {param_name}: {e}");
            0
        }
    }
}

/// Converts bytes to human-readable values
pub fn human_bytes(mut size: i64) -> String {
    let mut bytes = String::new();
    if size == 0 {
        return "0B".to_string();
    }
    if size < 0 {
        bytes.push_str("-");
        size = -size;
    }

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
