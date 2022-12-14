pub mod types;
pub mod utils;

use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::{bail, Context};
use color_eyre::Result;
use types::{CacheMiss, HeapSummary, LeakSummary};
use utils::{check_commands, parse_output_line, parse_output_line_f64, Capture};

use crate::types::{HeapSummaryHuman, LeakSummaryHuman};
use crate::utils::human_bytes;

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
    /// Check cache miss rates
    Cache(Cache),
}

#[derive(Args, Clone, Debug)]
pub struct Leak {}

#[derive(Args, Clone, Debug)]
pub struct Cache {}

#[derive(Args, Clone, Debug)]
pub struct Heap {
    /// Subtract bytes from total allocated
    #[clap(short, long, default_value_t = 0)]
    pub subtract_bytes: i64,
}

pub fn valgrind(
    bin: Option<String>,
    target_args: Vec<String>,
    valgrind_args: Vec<&str>,
) -> Result<String> {
    if cfg!(target_os = "windows") {
        bail!("Valgrind is not supported on Windows");
    }
    check_commands(&["valgrind"])?;
    let mut command = std::process::Command::new("valgrind");
    command.args(valgrind_args);
    command.args([format!("{}", bin.expect("provide a --bin <BIN>"))]);

    command.args(target_args);

    Ok(String::from_utf8(command.output()?.stderr)?)
}

pub fn cache(
    args: &Prof,
    _cache_args: &Cache,
    cargo_fn: Option<fn(bin: &Option<String>) -> Result<Option<String>>>,
) -> Result<()> {
    let mut bin = args.bin.clone();
    if let Some(cargo_fn) = cargo_fn {
        bin = cargo_fn(&args.bin)?;
    };
    let output = valgrind(bin, args.target_args.clone(), vec!["--tool=cachegrind"])?;

    let i1_cap =
        Capture::new(r"I1\s*miss rate:\s*([\d|\.]*)", &output).context("Cachegrind output")?;
    let mut l1i_miss = i1_cap.iter_next();

    let l2i_cap =
        Capture::new(r"L[L|2]i\s*miss rate:\s*([\d|\.]*)", &output).context("Cachegrind output")?;
    let mut lli_miss = l2i_cap.iter_next();

    let d1_cap =
        Capture::new(r"D1\s*miss rate:\s*([\d|\.]*)", &output).context("Cachegrind output")?;
    let mut l1d_miss = d1_cap.iter_next();

    let l2d_cap =
        Capture::new(r"L[L|2]d\s*miss rate:\s*([\d|\.]*)", &output).context("Cachegrind output")?;
    let mut lld_miss = l2d_cap.iter_next();

    let l2_cap =
        Capture::new(r"L[L|2]\s*miss rate:\s*([\d|\.]*)", &output).context("Cachegrind output")?;
    let mut ll_total_miss = l2_cap.iter_next();

    let cache_miss = CacheMiss {
        l1i: parse_output_line_f64("l1i miss rate", l1i_miss.next()),
        l1d: parse_output_line_f64("l1d miss rate", l1d_miss.next()),
        lli: parse_output_line_f64("lli miss rate", lli_miss.next()),
        lld: parse_output_line_f64("lld miss rate", lld_miss.next()),
        llt: parse_output_line_f64("ilt miss rate", ll_total_miss.next()),
    };

    if args.json {
        println!("{}", serde_json::to_string(&cache_miss)?);
    } else {
        println!("{}", serde_yaml::to_string(&cache_miss)?);
    }

    Ok(())
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
    let output = valgrind(bin, args.target_args.clone(), Vec::new())?;
    let exit_cap = Capture::new(r".*in use at exit\D*([\d|,]*)\D*([\d|,]*)", &output)
        .context("Valgrind output")?;
    let mut exit = exit_cap.iter_next();

    let total_cap = Capture::new(
        r".*total heap usage: ([\d|,]*)\D*([\d|,]*)\D*([\d|,]*)",
        &output,
    )
    .context("Valgrind output")?;
    let mut total = total_cap.iter_next();

    let heap_usage = HeapSummary {
        allocated_at_exit: parse_output_line("in use at exit", exit.next()),
        blocks_at_exit: parse_output_line("in use at exit blocks", exit.next()),
        allocations: parse_output_line("heap allocated", total.next()),
        frees: parse_output_line("heap frees", total.next()),
        allocated_total: parse_output_line("total heap usage", total.next())
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
    let res = valgrind(bin, args.target_args.clone(), Vec::new())?;

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
        definitely_lost: parse_output_line("definitely_lost", definite.next()),
        definitely_lost_blocks: parse_output_line("definitely_lost_blocks", definite.next()),

        indirectly_lost: parse_output_line("indirectly_lost", indirect.next()),
        indrectly_lost_blocks: parse_output_line("indirect_lost_blocks", indirect.next()),

        possibly_lost: parse_output_line("possibly_lost", possible.next()),
        possibly_lost_blocks: parse_output_line("possibly_lost_blocks", possible.next()),

        still_reachable: parse_output_line("still_reachable", reachable.next()),
        still_reachable_blocks: parse_output_line("still_reachable_blocks", reachable.next()),

        supressed: parse_output_line("supressed", suppressed.next()),
        supressed_blocks: parse_output_line("supressed_blocks", suppressed.next()),
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
