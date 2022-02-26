use anyhow::Context;
use clap::StructOpt;
use console::{style, Term};
use dialoguer::{
    theme::{ColorfulTheme, SimpleTheme, Theme},
    Confirm,
};
use makeclean::{projects_below, BuildToolManager, Cli, Command, Project, ProjectFilter};
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> anyhow::Result<()> {
    setup_logging();

    let cli = Cli::parse();

    let term = Term::stdout();
    let term_features = term.features();

    let color_theme = ColorfulTheme::default();
    let simple_theme = SimpleTheme {};
    let theme: &dyn Theme = if term_features.colors_supported() {
        &color_theme
    } else {
        &simple_theme
    };

    let use_json = match cli.subcommand {
        Command::List { json, .. } => json || !term_features.is_attended(),
        _ => false,
    };
    let paths_only = match cli.subcommand {
        Command::List { paths_only, .. } => paths_only,
        _ => false,
    };

    let mut projects = vec![];
    let project_filter = ProjectFilter::from(&cli);

    let mut build_tool_manager = BuildToolManager::default();
    let project_types = cli.project_types();
    if !project_types.is_empty() {
        build_tool_manager.filter(&project_types);
    }

    for project in projects_below(cli.directory(), &project_filter, &build_tool_manager) {
        // TODO only print if cleanable or --all is set
        println!("{}", format_project(&project, &term, use_json, paths_only));
        projects.push(project);
    }

    if paths_only {
        return Ok(());
    }

    match cli.subcommand {
        Command::Clean {
            dry_run, yes, zip, ..
        } => {
            println!();
            let do_continue = if dry_run || yes {
                true
            } else {
                Confirm::with_theme(theme)
                    .with_prompt("Clean up those projects?")
                    .default(true)
                    .interact()?
            };

            if do_continue {
                for project in &mut projects {
                    project
                        .clean(dry_run)
                        .with_context(|| format!("Failed to clean project {project}"))?;

                    if zip {
                        project
                            .zip(dry_run)
                            .with_context(|| format!("Failed to zip cleaned project {project}"))?;
                    }
                }
            }
        }
        Command::Zip { dry_run, yes, .. } => {
            println!();
            let do_continue = if dry_run || yes {
                true
            } else {
                Confirm::with_theme(theme)
                    .with_prompt("Zip each of those projects? (Note that if you want to clean them first, try `clean --zip` instead.)")
                    .default(true)
                    .interact()?
            };

            if do_continue {
                for project in &mut projects {
                    let zip_path = project
                        .zip(dry_run)
                        .with_context(|| format!("Failed to zip project {project}"))?;

                    println!("{zip_path}");
                }
            }
        }
        _ => {}
    };

    if term_features.is_attended() {
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

fn format_project(project: &Project, term: &Term, _use_json: bool, paths_only: bool) -> String {
    let term_features = term.features();
    let use_color = term_features.colors_supported() && term_features.is_attended() && !paths_only;

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

    let path = if use_color {
        if let Some(vcs_root) = project.vcs.as_ref().map(|vcs| vcs.root()) {
            // We'd expect the VCS root to be <= the project's own path.
            if let Ok(project_part) = project.path.strip_prefix(&vcs_root) {
                format!("{}{}", vcs_root, style(format!("/{}", project_part)).dim())
            } else {
                project.path.to_string()
            }
        } else {
            project.path.to_string()
        }
    } else {
        project.path.to_string()
    };

    if use_color {
        let info = style(format!("({}; {})", tools, vcs)).dim();
        format!("{} {}", path, info)
    } else if paths_only {
        path
    } else {
        format!("{} ({}; {})", path, tools, vcs)
    }
}

pub fn setup_logging() {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1")
    }
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .finish();
    subscriber.init();
}
