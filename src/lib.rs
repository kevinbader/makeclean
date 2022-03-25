pub mod build_tool_manager;
pub mod build_tools;
mod cli;
pub mod find_projects;
mod fs;
pub mod project;

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
    collections::{hash_map::Entry, HashMap, HashSet},
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

    // We use a Set as directories could overlap, and we don't want to print projects multiple times
    let mut printed_paths: HashSet<PathBuf> = HashSet::new();
    let mut freeable_bytes = 0;
    for directory in cli.directories {
        for project in projects_below(&directory, &project_filter, &build_tool_manager) {
            let is_new = printed_paths.insert(project.path.clone());
            if is_new {
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
        }
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

    // We use a HashMap as directories could overlap, and archiving a directory twice doesn't work
    let mut projects: HashMap<PathBuf, Project> = HashMap::new();
    for directory in cli.directories {
        for project in projects_below(&directory, &project_filter, &build_tool_manager) {
            if let Entry::Vacant(entry) = projects.entry(project.path.clone()) {
                print_project(&project, cli.json)?;
                entry.insert(project);
            }
        }
    }

    if cli.json && cli.dry_run {
        // If we'd continue, we'd fck up the JSON output, as the dry-run output
        // is not formatted.
        return Ok(());
    }

    let freeable_bytes = projects
        .values()
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
                for project in projects.values_mut() {
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
        let n_projects_without_vcs = projects.values().filter(|p| p.vcs.is_none()).count();
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
                .values()
                .filter(|p| p.vcs.is_none())
                .for_each(|p| println!("    {}", style(p.path.display()).dim()));
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

    let path = ProjectPath::from(project).render(use_color);

    let line = if use_color {
        let info = style(format!("({tools}; {vcs}; {mtime}{freeable})")).dim();
        format!("{} {}", path, info)
    } else {
        format!("{path} ({tools}; {vcs}; {mtime}{freeable})")
    };

    println!("{line}");

    Ok(())
}

struct ProjectPath {
    /// The path up to the project_part.
    parent: PathBuf,
    /// The path to the project directory within its Git repository, or the name of its
    /// directory otherwise.
    project: PathBuf,
}

impl ProjectPath {
    fn from(project: &Project) -> Self {
        // normalize, i.e., remove trailing slash
        let path: PathBuf = project.path.components().collect();

        if let Some(vcs_root) = project.vcs.as_ref().map(|vcs| vcs.root()) {
            match path.strip_prefix(&vcs_root) {
                Ok(postfix) if postfix == Path::new("") => {
                    // The project is at the root of its repository; we treat it as if there
                    // was no repository.
                }
                Ok(_) => {
                    // The project is within a parent repository/project. When displaying the
                    // project, this parent project should be considered part of the project.
                    let parent = vcs_root.parent().expect("is canonical path and not root");
                    // normalize, i.e., remove trailing slash
                    let project: PathBuf = path
                        .strip_prefix(parent)
                        .expect("is within parent")
                        .components()
                        .collect();
                    return ProjectPath {
                        parent: parent.to_owned(),
                        project,
                    };
                }
                Err(_) => panic!("expected the VCS root to be <= the project's own path"),
            }
        }

        let parent = path.parent().expect("is canonical path and not root");
        let project = path.strip_prefix(parent).expect("is within parent");
        ProjectPath {
            parent: parent.to_owned(),
            project: project.to_owned(),
        }
    }

    fn render(&self, use_color: bool) -> String {
        if use_color {
            format!(
                "{}/{}",
                self.parent.display(),
                style(self.project.display()).bold()
            )
        } else {
            format!("{}/{}", self.parent.display(), self.project.display())
        }
    }
}
