//! Command-line arguments.

#![deny(missing_docs)]

use camino::Utf8PathBuf;
use chrono::Duration;
use clap::Parser;
use regex::Regex;

use crate::{project::ProjectStatus, ProjectFilter};

/// Removes generated and downloaded files from code projects to free up space.
///
/// Only supports Git-controlled projects (other projects are ignored).
#[derive(Parser, Debug)]
pub struct Cli {
    /// JSON output (default when stdout is piped)
    #[clap(long)]
    pub json: bool,

    /// Lists projects only.
    #[clap(short, long)]
    pub list: bool,

    /// Projects that were modified more recently than this are ignored.
    /// Examples: 1d = a day, 2w = two weeks, 1m = a month, 1y = a year.
    #[clap(short, long, parse(try_from_str=parse_duration), global(true))]
    pub min_age: Option<Duration>,

    /// Only consider projects that use the given build tool.
    /// Use more than once for multiple project types.
    /// By default, all known project types are considered.
    #[clap(short = 't', long = "type")]
    pub types: Vec<String>,

    /// Dry run - prints what would happen but doesn't actually remove/change anything.
    #[clap(short = 'n', long)]
    pub dry_run: bool,

    /// Automatically execute without asking, i.e., skipping the prompt.
    #[clap(long)]
    pub yes: bool,

    /// Additionally compress cleaned projects.
    #[clap(short = 'z', long)]
    pub archive: bool,

    /// Recursively searches for project in this directory
    #[clap(default_value = ".", global(true))]
    pub directory: Utf8PathBuf,
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

impl From<&Cli> for ProjectFilter {
    fn from(cli: &Cli) -> Self {
        // --min-age has a different default, depending on whether we just list
        // projects or we actually want to clean them.
        let min_age = cli.min_age.unwrap_or_else(|| {
            if cli.list {
                Duration::zero()
            } else {
                Duration::days(30)
            }
        });

        let status = if cli.list {
            ProjectStatus::Any
        } else {
            ProjectStatus::ExceptClean
        };

        ProjectFilter { min_age, status }
    }
}
