use std::{ops::Deref, process::Command};

use color_eyre::{
    eyre::{eyre, Context},
    Help, Result,
};
use regex::{Captures, Match, SubCaptureMatches};
use tracing::warn;

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

pub fn warn_and_return(param_name: &str) -> i64 {
    warn!("{} not found in valgrind output", param_name);
    0
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

pub fn parse_output_line(param_name: &str, param: Option<Option<Match>>) -> i64 {
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

/// Wrap a standard capture with a custom `new()` implmentation to de-depulicate code
pub struct Capture<'a>(Captures<'a>);

impl<'a> Capture<'a> {
    pub fn new(re: &str, output: &'a str) -> Result<Self> {
        let re = regex::Regex::new(re).expect("failed to compile regex");
        match re.captures(output) {
            Some(x) => Ok(Capture(x)),
            None => Err(eyre!("No match found for regex: {}", re)),
        }
    }
    /// Create an iterator over the captures but skip the fist one
    pub fn iter_next(&'a self) -> SubCaptureMatches<'a, 'a> {
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
