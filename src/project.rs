mod archive;
mod clean;
pub mod dto;
mod mtime;

use crate::{
    build_tool_manager::BuildToolManager,
    build_tools::{BuildStatus, BuildTool},
    vcs::VersionControlSystem,
};
use anyhow::format_err;
use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Duration, Local, Utc};
use std::fmt;
use tracing::{trace, warn};

use self::mtime::dir_mtime;

#[derive(Debug)]
pub struct Project {
    /// The name of project.
    ///
    /// Typically what the first build tool thinks it's called or the name of
    /// the enclosing folder.
    pub name: String,
    /// Where this project is located.
    pub path: Utf8PathBuf,
    /// The build tools used.
    pub build_tools: Vec<Box<dyn BuildTool>>,
    /// The VCS, if under version control.
    pub vcs: Option<VersionControlSystem>,
    /// When this project was last modified (most recent commit timestamp).
    pub mtime: DateTime<Utc>,
}

impl Project {
    pub fn from_dir(
        path: &Utf8Path,
        project_filter: &ProjectFilter,
        vcs: Option<VersionControlSystem>,
        build_tool_manager: &BuildToolManager,
    ) -> Option<Project> {
        // Is this a project? => yes, if at least one build tool is recognized
        let build_tools = build_tool_manager.probe(path);
        if build_tools.is_empty() {
            return None;
        }

        if let ProjectStatus::ExceptClean = project_filter.status {
            // Ignore this project if _all_ build tools report a clean state
            if build_tools
                .iter()
                .all(|tool| matches!(tool.status(), Ok(BuildStatus::Clean)))
            {
                return None;
            }
        }

        let project_name = match project_name_from(path, &build_tools) {
            Ok(name) => name,
            Err(e) => {
                warn!("Failed to determine project name for {path:?}: {e}");
                return None;
            }
        };

        let mtime =
            dir_mtime(path.as_std_path()).expect("BUG: build tool recognized but no files?!");
        let mtime: DateTime<Local> = mtime.into();
        let mtime: DateTime<Utc> = mtime.into();

        if Utc::now().signed_duration_since(mtime) < project_filter.min_stale {
            trace!(
                ?path,
                %mtime,
                min_stale=%project_filter.min_stale,
                "Project skipped due to recent mtime",
            );
            return None;
        }

        Some(Project {
            name: project_name,
            path: path.to_owned(),
            build_tools,
            vcs,
            mtime,
        })
    }
}

fn project_name_from(
    path: &Utf8Path,
    build_tools: &[Box<dyn BuildTool>],
) -> anyhow::Result<String> {
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
    Ok(dirname.to_string())
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
        write!(f, "{} ({}; VCS: {})", path, tools, vcs)
    }
}

#[derive(Debug)]
pub struct ProjectFilter {
    pub min_stale: Duration,
    pub status: ProjectStatus,
}

impl Default for ProjectFilter {
    fn default() -> Self {
        Self {
            min_stale: Duration::days(0),
            status: ProjectStatus::ExceptClean,
        }
    }
}

#[derive(Debug)]
pub enum ProjectStatus {
    Any,
    ExceptClean,
}
