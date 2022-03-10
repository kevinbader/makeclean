pub mod build_tool_manager;
pub mod build_tools;
mod cli;
pub mod find_projects;
mod fs;
pub mod project;
mod vcs;

use anyhow::Context;
use build_tool_manager::BuildToolManager;
use chrono::Duration;
use clap::{CommandFactory, ErrorKind};
use console::{colors_enabled, style};
use dialoguer::{
    theme::{ColorfulTheme, SimpleTheme, Theme},
    Confirm,
};
use project::Project;
use std::{
    io,
    path::{Path, PathBuf},
};
use tracing::debug;

pub use crate::cli::Cli;
use crate::{
    find_projects::projects_below,
    fs::format_size,
    project::{dto::ProjectDto, ProjectFilter, StatusFilter},
};

/// Implementation of `makeclean --list`
pub fn list(cli: Cli, build_tool_manager: BuildToolManager) -> anyhow::Result<()> {
    let project_filter = {
        let min_stale = cli.min_stale.unwrap_or_else(Duration::zero);
        let status = StatusFilter::Any;
        ProjectFilter { min_stale, status }
    };
    debug!("listing projects with {project_filter:?}");

    let mut freeable_bytes = 0;
    for project in projects_below(&cli.directory, &project_filter, &build_tool_manager) {
        print_project(&project, cli.json)?;
        freeable_bytes += project
            .build_tools
            .iter()
            .map(|x| match x.status() {
                Ok(build_tools::BuildStatus::Built { freeable_bytes }) => freeable_bytes,
                _ => 0,
            })
            .sum::<u64>();
    }

    if !cli.json {
        println!();
        let message = format!(
            "Found {} of build artifacts and dependencies.",
            format_size(freeable_bytes)
        );
        if colors_enabled() {
            println!("{}", style(message).green());
        } else {
            println!("{}", message);
        }
    }

    Ok(())
}

/// Removes generated and downloaded files from code projects to free up space.
///
/// Runs in interactive mode unless either one of `cli.dry_run` and `cli.yes` is true.
pub fn clean(cli: Cli, build_tool_manager: BuildToolManager) -> anyhow::Result<()> {
    // I couldn't figure out how to do this with Clap..
    if cli.json && !cli.dry_run && !cli.yes {
        // Would be interactive mode, which doesn't make sense with JSON - the
        // prompt is not JSON formatted, after all.
        let mut cmd = Cli::command();
        cmd.error(
            ErrorKind::ArgumentConflict,
            "With `--json`, either `--dry-run` or `--yes` is required.",
        )
        .exit();
    }

    let project_filter = {
        let min_stale = cli.min_stale.unwrap_or_else(|| Duration::days(30));
        let status = if cli.archive {
            StatusFilter::Any
        } else {
            StatusFilter::ExceptClean
        };
        ProjectFilter { min_stale, status }
    };

    let mut projects = vec![];
    for project in projects_below(&cli.directory, &project_filter, &build_tool_manager) {
        print_project(&project, cli.json)?;
        projects.push(project);
    }

    if cli.json && cli.dry_run {
        // If we'd continue, we'd fck up the JSON output, as the dry-run output
        // is not formatted.
        return Ok(());
    }

    let freeable_bytes = projects
        .iter()
        .flat_map(|p| p.build_tools.iter())
        .map(|bt| match bt.status() {
            Ok(build_tools::BuildStatus::Built { freeable_bytes }) => freeable_bytes,
            _ => 0,
        })
        .sum::<u64>();

    let has_cleaned = {
        if projects.is_empty() {
            false
        } else {
            let do_continue = if cli.dry_run {
                println!("\n{}", style("WOULD DO:").bold());
                true
            } else if cli.yes {
                true
            } else {
                println!();

                let theme = theme();
                let prompt = format!("Clean up those projects ({})?", format_size(freeable_bytes));
                Confirm::with_theme(&*theme)
                    .with_prompt(prompt)
                    .default(true)
                    .interact()?
            };

            if do_continue {
                for project in &mut projects {
                    project
                        .clean(cli.dry_run)
                        .with_context(|| format!("Failed to clean project {project}"))?;

                    if cli.archive {
                        project.archive(cli.dry_run).with_context(|| {
                            format!("Failed to archive cleaned project {project}")
                        })?;
                    }
                }

                !cli.dry_run
            } else {
                println!("No changes made.");
                false
            }
        }
    };

    if !cli.json {
        println!();
        println!("{}", style("SUMMARY:").bold());
        let projects_label = if projects.len() == 1 {
            "project"
        } else {
            "projects"
        };
        println!(
            "  {}",
            style(if has_cleaned {
                format!(
                    "{} {projects_label} cleaned, which freed approx. {} of build artifacts and dependencies.",
                    projects.len(),
                    format_size(freeable_bytes)
                )
            } else {
                format!(
                    "{} built {projects_label} found, with {} of build artifacts and dependencies.",
                    projects.len(),
                    format_size(freeable_bytes)
                )
            })
            .green()
        );
        let n_projects_without_vcs = projects.iter().filter(|p| p.vcs.is_none()).count();
        if n_projects_without_vcs > 0 {
            println!(
                "  {}",
                style(format!(
                    "{} projects not under version control:",
                    n_projects_without_vcs
                ))
                .red()
            );
            projects
                .iter()
                .filter(|p| p.vcs.is_none())
                .for_each(|p| println!("    {}", style(p.path.as_str()).dim()));
        }
    }

    Ok(())
}

fn theme() -> Box<dyn Theme> {
    if colors_enabled() {
        Box::new(ColorfulTheme::default())
    } else {
        Box::new(SimpleTheme {})
    }
}

fn print_project(project: &Project, json: bool) -> anyhow::Result<()> {
    if json {
        let dto = ProjectDto::from(project);
        serde_json::to_writer(io::stdout(), &dto)?;
        // Add the newline:
        println!();
    } else {
        pretty_print_project(project)?;
    }
    Ok(())
}

fn pretty_print_project(project: &Project) -> anyhow::Result<()> {
    let use_color = colors_enabled();

    let tools = project
        .build_tools
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let vcs = project
        .vcs
        .as_ref()
        .map(|x| x.name())
        .unwrap_or_else(|| "no VCS");
    let freeable = match project.freeable_bytes() {
        0 => String::new(),
        bytes => format!("; {}", format_size(bytes)),
    };
    // See https://docs.rs/chrono/latest/chrono/format/strftime/index.html
    let mtime = project.mtime.format("%F");

    let path = match (use_color, path_components(project)) {
        (false, (project, None)) => project,
        (false, (repo, Some(project))) => format!("{repo}{project}"),
        (true, (project, None)) => project,
        (true, (repo, Some(project))) => format!("{}{}", repo, style(project).bold()),
    };

    let line = if use_color {
        let info = style(format!("({tools}; {vcs}; {mtime}{freeable})")).dim();
        format!("{} {}", path, info)
    } else {
        format!("{path} ({tools}; {vcs}; {mtime}{freeable})")
    };

    println!("{line}");

    Ok(())
}

/// Returns path to project and path to subproject if within parent project
fn path_components(project: &Project) -> (String, Option<String>) {
    // normalize, i.e., remove trailing slash
    let path: PathBuf = project.path.components().collect();

    if let Some(mut vcs_root) = project.vcs.as_ref().map(|vcs| vcs.root()) {
        match path.strip_prefix(&vcs_root) {
            Ok(prefix) if prefix == Path::new("") => {
                // The project is at the root of its repository
                (path.display().to_string(), None)
            }
            Ok(project_part) => {
                // The repo path should have a trailing slash, so we push a component, just to make sure
                vcs_root.push("");
                let vcs_root = vcs_root.as_str().to_owned();

                // The project part should not have a trailing slash, so we normalize again
                let project_part: PathBuf = project_part.components().collect();
                let project_part = project_part.display().to_string();

                (vcs_root, Some(project_part))
            }
            Err(_) => panic!("expected the VCS root to be <= the project's own path"),
        }
    } else {
        (path.display().to_string(), None)
    }
}
