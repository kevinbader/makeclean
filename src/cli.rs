//! Command-line arguments.

#![deny(missing_docs)]

use std::path::PathBuf;

use clap::Parser;
use regex::Regex;
use time::Duration;

use crate::build_tools::BuildToolKind;

/// Options
#[derive(Parser, Debug)]
#[clap(version, about, long_about=None)]
pub struct Cli {
    /// JSON output (default when stdout is piped)
    ///
    /// When present, either `--dry-run` or `--yes` is required.
    #[clap(long)]
    pub json: bool,

    /// Lists projects only.
    #[clap(short, long)]
    pub list: bool,

    /// Projects that were modified more recently than this are ignored.
    /// With `--list`, this defaults to 0, otherwise to one month.
    /// Examples: 1d = a day, 2w = two weeks, 1m = a month, 1y = a year.
    #[clap(value_name(r"DURATION"), short, long, parse(try_from_str=parse_duration))]
    pub min_stale: Option<Duration>,

    /// Only consider projects that use the given build tool.
    /// Use more than once for multiple project types.
    /// By default, all known project types are considered.
    ///
    /// For example, to consider Cargo and NPM projects:
    ///
    /// makeclean -t cargo -t npm
    #[clap(short = 't', long = "type", arg_enum)]
    pub types: Vec<BuildToolKind>,

    /// Dry run - prints what would happen but doesn't actually remove/change anything.
    #[clap(short = 'n', long)]
    pub dry_run: bool,

    /// Automatically execute without asking, i.e., skipping the prompt.
    #[clap(long)]
    pub yes: bool,

    /// Additionally compress cleaned projects.
    ///
    /// After cleaning a project, its contents are moved into a tar.xz file. To
    /// restore the project, use `tar` (which is probably already installed on
    /// your system):
    ///
    /// cd path/to/project && tar -xaf project-name.tar.xz && rm project-name.tar.xz
    #[clap(short = 'z', long)]
    pub archive: bool,

    /// Recursively searches for project in these directories
    #[clap(default_value = ".")]
    pub directories: Vec<PathBuf>,
}

fn parse_duration(s: &str) -> anyhow::Result<Duration> {
    let captures = Regex::new(r"^(?P<n>\d+)(?P<unit>[dDwWmMyY])?$")
        .unwrap()
        .captures(s)
        .ok_or_else(|| {
            anyhow::format_err!(
                r#"Cannot parse {:?}. Try "1d" for a day, "1w" for a week, "1m" for a month or "1y" for a year."#,
                s
            )
        })?;
    let n = captures
        .name("n")
        .and_then(|n| n.as_str().parse::<i64>().ok())
        .unwrap();
    assert!(n >= 0, "Duration cannot be negative");
    let unit = captures.name("unit").map(|u| u.as_str()).unwrap_or("d");
    let duration = match unit {
        "d" | "D" => Duration::days(n),
        "w" | "W" => Duration::weeks(n),
        "m" | "M" => Duration::days(7 * n * 52 / 12),
        "y" | "Y" => Duration::weeks(n * 52),
        _ => unreachable!("the regex should make sure of that"),
    };
    Ok(duration)
}
