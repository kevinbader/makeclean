//! Represents a software project.

mod archive;
mod clean;
pub mod dto;
mod mtime;
mod vcs;

use crate::{
    build_tool_manager::BuildToolManager,
    build_tools::{BuildStatus, BuildTool},
};
use anyhow::format_err;
use std::{
    fmt,
    path::{Path, PathBuf},
};
use time::{Duration, OffsetDateTime};
use tracing::{trace, warn};

use self::{mtime::dir_mtime, vcs::VersionControlSystem};

/// Main entity.
#[derive(Debug)]
pub struct Project {
    /// The name of project.
    ///
    /// Typically what the first build tool thinks it's called or the name of
    /// the enclosing folder.
    pub name: String,
    /// Where this project is located.
    pub path: PathBuf,
    /// The build tools used.
    pub build_tools: Vec<Box<dyn BuildTool>>,
    /// The VCS, if under version control.
    pub vcs: Option<VersionControlSystem>,
    /// When this project was last modified (most recent commit timestamp).
    pub mtime: OffsetDateTime,
}

impl Project {
    pub fn from_dir(
        path: &Path,
        project_filter: &ProjectFilter,
        build_tool_manager: &BuildToolManager,
    ) -> anyhow::Result<Option<Project>> {
        // Is this a project? => yes, if at least one build tool is recognized
        let build_tools = build_tool_manager.probe(path);
        if build_tools.is_empty() {
            return Ok(None);
        }

        if let StatusFilter::ExceptClean = project_filter.status {
            // Ignore this project if _all_ build tools report a clean state
            if build_tools
                .iter()
                .all(|tool| matches!(tool.status(), Ok(BuildStatus::Clean)))
            {
                return Ok(None);
            }
        }

        let project_name = match project_name_from(path, &build_tools) {
            Ok(name) => name,
            Err(e) => {
                warn!("Failed to determine project name for {path:?}: {e}");
                return Ok(None);
            }
        };

        let mtime = dir_mtime(path)
            .ok_or_else(|| format_err!("BUG: build tool recognized but no files?!"))?;
        let now = OffsetDateTime::now_utc();

        if (now - mtime) < project_filter.min_stale {
            trace!(
                ?path,
                %mtime,
                min_stale=%project_filter.min_stale,
                "Project skipped due to recent mtime",
            );
            return Ok(None);
        }

        let vcs = VersionControlSystem::try_from(path)?;

        Ok(Some(Project {
            name: project_name,
            path: path.to_owned(),
            build_tools,
            vcs,
            mtime,
        }))
    }

    pub fn freeable_bytes(&self) -> u64 {
        self.build_tools
            .iter()
            .map(|x| match x.status() {
                Ok(BuildStatus::Built { freeable_bytes }) => freeable_bytes,
                _ => 0,
            })
            .sum::<u64>()
    }
}

fn project_name_from(path: &Path, build_tools: &[Box<dyn BuildTool>]) -> anyhow::Result<String> {
    for tool in build_tools {
        match tool.project_name() {
            Some(Ok(name)) => return Ok(name),
            Some(Err(e)) => return Err(e),
            None => continue,
        }
    }
    // None of the build tools knows the project's name, so let's use the
    // directory name as a fallback
    let dirname = path.components().last().ok_or_else(|| {
        format_err!(
            "Could not determine project name: Could not determine the directory name of {path:?}"
        )
    })?;
    Ok(dirname.as_os_str().to_string_lossy().to_string())
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = &self.path;
        let tools = self
            .build_tools
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let vcs = self
            .vcs
            .as_ref()
            .map(|x| format!("{x:?}"))
            .unwrap_or_else(|| "none".to_owned());
        write!(f, "{} ({}; VCS: {})", path.display(), tools, vcs)
    }
}

/// Defines which projects to consider
#[derive(Debug)]
pub struct ProjectFilter {
    /// Projects that were modified more recently than this are ignored.
    pub min_stale: Duration,
    /// Projects that don't satisfy the status filter are ignored
    pub status: StatusFilter,
}

/// Filter by status reported by the [`Project`]'s build tools.
#[derive(Debug)]
pub enum StatusFilter {
    /// Any project is considered
    Any,
    /// All projects that are not already clean are considered
    ExceptClean,
}
