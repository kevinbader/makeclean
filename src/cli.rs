//! Command-line arguments.

#![deny(missing_docs)]

use camino::{Utf8Path, Utf8PathBuf};
use chrono::Duration;
use clap::{Args, Parser, Subcommand};
use regex::Regex;

use crate::{project::ProjectStatus, ProjectFilter};

/// Removes generated and downloaded files from code projects to free up space.
///
/// Only supports Git-controlled projects (other projects are ignored).
#[derive(Parser, Debug)]
pub struct Cli {
    /// The action to take.
    #[clap(subcommand)]
    pub subcommand: Command,
}

impl Cli {
    /// Build-tool filter
    pub fn project_types(&self) -> Vec<String> {
        let types = match self.subcommand {
            Command::List { ref common, .. } => &common.project_type_filter,
            Command::Clean { ref common, .. } => &common.project_type_filter,
            Command::Zip { ref common, .. } => &common.project_type_filter,
        };
        types.iter().map(|s| s.to_lowercase()).collect()
    }

    /// Recursively searches for project in this directory
    pub fn directory(&self) -> &Utf8Path {
        match self.subcommand {
            Command::List { ref common, .. } => &common.directory,
            Command::Clean { ref common, .. } => &common.directory,
            Command::Zip { ref common, .. } => &common.directory,
        }
    }
}

/// Subcommand
#[derive(Subcommand, Debug)]
pub enum Command {
    /// List and filter projects
    List {
        /// Common options
        #[clap(flatten)]
        common: CommonOptions,

        /// List all projects, not just those that could be cleaned up
        #[clap(short, long)]
        any_status: bool,

        /// JSON output (default when stdout is piped)
        #[clap(long)]
        json: bool,

        /// Only print paths
        #[clap(long)]
        paths_only: bool,
    },
    /// Remove dependency and build caches, as well as build artefacts.
    Clean {
        /// Common options
        #[clap(flatten)]
        common: CommonOptions,

        /// Also compress the project contents into a zip file.
        #[clap(short, long)]
        zip: bool,

        /// Dry run - prints what would happen but doesn't actually remove anything.
        #[clap(short = 'n', long)]
        dry_run: bool,

        /// Automatically execute, without asking, i.e., skipping the prompt.
        #[clap(long)]
        yes: bool,
    },
    /// Compress the projects' contents into zip files
    Zip {
        /// Common options
        #[clap(flatten)]
        common: CommonOptions,

        /// Dry run - prints what would happen but doesn't actually create/remove anything.
        #[clap(short = 'n', long)]
        dry_run: bool,

        /// Automatically execute, without asking, i.e., skipping the prompt.
        #[clap(long)]
        yes: bool,
    },
}

#[derive(Args, Debug)]
pub struct CommonOptions {
    /// Projects that were modified more recently than this are ignored.
    /// Examples: 1d = a day, 2w = two weeks, 1m = a month, 1y = a year.
    #[clap(short, long, parse(try_from_str=parse_duration), default_value="1m", global(true))]
    pub min_age: Duration,

    /// Only consider projects that use the given build tool.
    /// Use more than once for multiple project types.
    /// By default, all known project types are considered.
    #[clap(short = 't', long = "type")]
    pub project_type_filter: Vec<String>,

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
        let min_age = match cli.subcommand {
            Command::List { ref common, .. } => common.min_age,
            Command::Clean { ref common, .. } => common.min_age,
            Command::Zip { ref common, .. } => common.min_age,
        };
        let status = match cli.subcommand {
            Command::List {
                any_status: true, ..
            } => ProjectStatus::Any,
            Command::Zip { .. } => ProjectStatus::Any,
            _ => ProjectStatus::ExceptClean,
        };
        ProjectFilter { min_age, status }
    }
}

// /// Removes generated and downloaded files from code projects to free up space.
// ///
// /// Only supports Git-controlled projects (other projects are ignored).
// #[derive(Parser, Debug)]
// pub struct Cli {
//     /// Dry run - prints what would happen but doesn't actually remove anything.
//     #[clap(short = 'n', long)]
//     pub dry_run: bool,
//
//     // // /// The format used for printing the results (ignored in interactive mode)
//     // // #[clap(short, long, arg_enum, default_value_t = OutputFormat::Table)]
//     // // pub format: OutputFormat,
//     // //
//     // /// Interactively choose for each project whether to clean or ignore it.
//     // #[clap(short, long)]
//     // pub interactive: bool,
//     //
//     /// Automatically cleans all matching projects, skipping the user prompt.
//     #[clap(long)]
//     pub clean: bool,
//
//     /// Automatically archives all matching projects, skipping the user prompt.
//     ///
//     /// Is not implied by `--clean`, that is, `--archive` without `--clean`
//     /// creates archives that might include dependency and build artefacts.
//     #[clap(long)]
//     pub archive: bool,
//
//     /// Marie Kondo flavored interactive mode.
//     #[clap(short, long)]
//     pub mariekondo: bool,
//
//     // /// Removes files automatically, skipping the prompt
//     // #[clap(long)]
//     // pub yes: bool,
//     //
//     /// Projects that were modified more recently than this are ignored (in days)
//     #[clap(long, default_value_t = 30)]
//     pub min_age: u32,
//
//     /// Recursively searches for project in this directory
//     #[clap(default_value = ".")]
//     pub directory: Utf8PathBuf,
// }
//
// /// The format used for printing the results.
// #[derive(Debug, Clone, ArgEnum)]
// pub enum OutputFormat {
//     /// A human-readable, table-like format
//     Table,
//     /// A not-so-human-readable JSON format
//     Json,
// }
