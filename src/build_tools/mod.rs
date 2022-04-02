use std::{fs, path::Path};

use clap::ArgEnum;
use displaydoc::Display;

use crate::fs::dir_size;

pub mod cargo;
pub mod elm;
pub mod flutter;
pub mod gradle;
pub mod maven;
pub mod mix;
pub mod npm;

#[derive(Debug, Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum BuildToolKind {
    /// Cargo
    Cargo,
    /// Rust
    Rust,
    /// rs
    Rs,

    /// Elm
    Elm,

    /// Flutter
    Flutter,

    /// Gradle
    Gradle,

    /// Maven
    Maven,
    /// mvn
    Mvn,

    /// Mix
    Mix,
    /// Elixir
    Elixir,
    /// ex
    Ex,
    /// exs
    Exs,

    /// NPM
    Npm,
}

pub trait BuildToolProbe: std::fmt::Debug {
    /// Returns a [`BuildTool`] instance if configured in the given directory.
    fn probe(&self, dir: &Path) -> Option<Box<dyn BuildTool>>;

    /// Whether the build tool matches a given build tool name or project type.
    fn applies_to(&self, kind: BuildToolKind) -> bool;
}

pub trait BuildTool: std::fmt::Debug + std::fmt::Display {
    /// Clean the project.
    ///
    /// Depending on the build tool represented, this should preferably invoke
    /// the tool itself, calling its "clean" command. If that's not possible,
    /// because the tool either doesn't support it (e.g., NPM), or because the
    /// tool is not present/installed, the implementation may fall back to
    /// removing well-known directories itself (e.g., the `node_modules`
    /// directory for NPM).
    ///
    /// If `dry_run` is true, no files are changed. Instead, a description on
    /// what would happen is printed to stdout.
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()>;

    fn status(&self) -> anyhow::Result<BuildStatus> {
        Ok(BuildStatus::Unknown)
    }

    /// The project's name as parsed from build tool configuration.
    ///
    /// Returns None if the project has no name configured, or in case the build
    /// tool doesn't implement this feature yet. As a fallback, the name of the
    /// parent directory will be considered as the project's name.
    fn project_name(&self) -> Option<anyhow::Result<String>> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BuildStatus {
    /// There are no build artifacts or dependency that could be cleaned up.
    Clean,
    /// The project could be cleaned up, potentially freeing up `freeable_bytes`
    /// bytes.
    Built { freeable_bytes: u64 },
    /// The status cannot be determined.
    Unknown,
}

//
// Utils for build tools
//

fn remove_dirs(project_dir: &Path, ephemeral_dirs: &[&str], dry_run: bool) -> anyhow::Result<()> {
    for dir in ephemeral_dirs
        .iter()
        .map(|dirname| project_dir.join(dirname))
        .filter(|dir| dir.is_dir())
    {
        if dry_run {
            println!("rm -r '{}'", dir.display());
        } else {
            fs::remove_dir_all(dir)?;
        }
    }

    Ok(())
}

fn status_from_dirs(project_dir: &Path, ephemeral_dirs: &[&str]) -> anyhow::Result<BuildStatus> {
    let size: u64 = ephemeral_dirs
        .iter()
        .map(|dirname| project_dir.join(dirname))
        .filter(|dir| dir.is_dir())
        .map(|dir| dir_size(&dir))
        .sum();

    let status = match size {
        0 => BuildStatus::Clean,
        freeable_bytes => BuildStatus::Built { freeable_bytes },
    };

    Ok(status)
}
