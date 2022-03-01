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
use console::{style, Term};
use dialoguer::{
    theme::{ColorfulTheme, SimpleTheme, Theme},
    Confirm,
};
use project::Project;
use std::io;
use tracing::debug;

pub use crate::cli::Cli;
use crate::{
    find_projects::projects_below,
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

    for project in projects_below(&cli.directory, &project_filter, &build_tool_manager) {
        print_project(&project, cli.json)?;
    }

    Ok(())
}

/// Removes generated and downloaded files from code projects to free up space.
///
/// Runs in interactive mode unless either one of `cli.dry_run` and `cli.yes` is true.
pub fn clean(cli: Cli, build_tool_manager: BuildToolManager) -> anyhow::Result<()> {
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

    if projects.is_empty() {
        // TODO: Perhaps output "No projects found. Try running with RUST_LOG=trace to see why."
        // This will fail tests, which currently expect no output besides projects. A `--quiet` switch should help.

        return Ok(());
    }

    println!();
    let do_continue = if cli.dry_run || cli.yes {
        true
    } else {
        let theme = theme();
        Confirm::with_theme(&*theme)
            .with_prompt("Clean up those projects?")
            .default(true)
            .interact()?
    };

    if do_continue {
        for project in &mut projects {
            project
                .clean(cli.dry_run)
                .with_context(|| format!("Failed to clean project {project}"))?;

            if cli.archive {
                project
                    .archive(cli.dry_run)
                    .with_context(|| format!("Failed to archive cleaned project {project}"))?;
            }
        }
    }

    if Term::stdout().features().is_attended() {
        println!();
        println!(
            "{}",
            style(format!("{} projects found.", projects.len())).green()
        );
        // TODO print how many of those could be (OR HAVE BEEN) cleaned and how much space that would/was save/d.
        let n_projects_without_vcs = projects.iter().filter(|p| p.vcs.is_none()).count();
        if n_projects_without_vcs > 0 {
            println!(
                "{}",
                style(format!(
                    "{} projects not under version control:",
                    n_projects_without_vcs
                ))
                .red()
            );
            projects
                .iter()
                .filter(|p| p.vcs.is_none())
                .for_each(|p| println!("  {}", style(p.path.as_str()).dim()));
        }
    }

    Ok(())
}

fn theme() -> Box<dyn Theme> {
    if Term::stdout().features().colors_supported() {
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
        println!("{project}");
    }
    Ok(())
}

// fn format_project(project: &Project, term: &Term, _use_json: bool) -> String {
//     let term_features = term.features();
//     let use_color = term_features.colors_supported() && term_features.is_attended() && !paths_only;
//
//     let tools = project
//         .build_tools
//         .iter()
//         .map(|x| x.to_string())
//         .collect::<Vec<_>>()
//         .join(", ");
//     let vcs = project
//         .vcs
//         .as_ref()
//         .map(|x| x.name())
//         .unwrap_or_else(|| "no VCS");
//
//     let path = if use_color {
//         if let Some(vcs_root) = project.vcs.as_ref().map(|vcs| vcs.root()) {
//             // We'd expect the VCS root to be <= the project's own path.
//             if let Ok(project_part) = project.path.strip_prefix(&vcs_root) {
//                 format!("{}{}", vcs_root, style(format!("/{}", project_part)).dim())
//             } else {
//                 project.path.to_string()
//             }
//         } else {
//             project.path.to_string()
//         }
//     } else {
//         project.path.to_string()
//     };
//
//     if use_color {
//         let info = style(format!("({}; {})", tools, vcs)).dim();
//         format!("{} {}", path, info)
//     } else {
//         format!("{} ({}; {})", path, tools, vcs)
//     }
// }
