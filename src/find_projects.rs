use crate::{
    build_tool_manager::BuildToolManager, fs::canonicalized, project::ProjectFilter,
    vcs::VersionControlSystem, Project,
};
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::VecDeque;
use tracing::{debug, trace, warn};

pub fn projects_below<'a>(
    path: &Utf8Path,
    project_filter: &'a ProjectFilter,
    build_tool_manager: &'a BuildToolManager,
) -> impl Iterator<Item = Project> + 'a {
    let path = canonicalized(path).unwrap();
    Iter::new(path, project_filter, build_tool_manager)
}

pub struct Iter<'a> {
    queue: VecDeque<Utf8PathBuf>,
    project_filter: &'a ProjectFilter,
    build_tool_manager: &'a BuildToolManager,
}

impl<'a> Iter<'a> {
    fn new(
        path: Utf8PathBuf,
        project_filter: &'a ProjectFilter,
        probes: &'a BuildToolManager,
    ) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(path);

        Self {
            queue,
            project_filter,
            build_tool_manager: probes,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Project;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.queue.is_empty() {
                return None;
            }
            match process_next_dir(
                &mut self.queue,
                self.project_filter,
                self.build_tool_manager,
            ) {
                Ok(Some(project)) => {
                    debug!("project found at {:?}", project.path);
                    return Some(project);
                }
                Ok(None) => {}
                Err(e) => warn!("{e}"),
            };
            // If no project was found, we keep going.
        }
    }
}

fn process_next_dir(
    queue: &mut VecDeque<Utf8PathBuf>,
    project_filter: &ProjectFilter,
    build_tool_manager: &BuildToolManager,
) -> anyhow::Result<Option<Project>> {
    assert!(!queue.is_empty());
    let path = queue.pop_front().unwrap();
    trace!("--> {path}");

    let mut entries =
        std::fs::read_dir(&path).with_context(|| format!("Failed to read directory {path}"))?;

    let vcs = VersionControlSystem::try_from(&path)?;

    // This check really only catches the top-level directory, as the
    // subdirectories are checked for this before being added to the queue
    if let Some(ref vcs) = vcs {
        if vcs.is_path_ignored(&path) {
            debug!(?path, "skipping directory ignored by VCS");
            trace!("<-- {path}");
            return Ok(None);
        }
    }

    // We ignore any IO errors for individual directory entries
    while let Some(Ok(entry)) = entries.next() {
        let path = entry.path();
        // Only consider directories:
        if !path.is_dir() {
            continue;
        }
        let path = Utf8PathBuf::try_from(path.canonicalize()?)?;
        if is_hidden(&path) || is_special_dir(&path) {
            continue;
        }
        if let Some(ref vcs) = vcs {
            if vcs.is_path_ignored(&path) {
                debug!(?path, "skipping directory ignored by VCS");
                continue;
            }
        }
        trace!(?path, "queued");
        queue.push_back(path);
    }

    let project = Project::from_dir(&path, project_filter, vcs, build_tool_manager);
    trace!("<-- {path}");
    Ok(project)
}

fn is_hidden(path: &Utf8Path) -> bool {
    path.file_name()
        .map(|fname| fname.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn is_special_dir(path: &Utf8Path) -> bool {
    let user = std::env::var("USER").unwrap();
    path == Utf8PathBuf::try_from(format!("/Users/{}/Library", user)).unwrap()
}
#[cfg(not(target_os = "macos"))]
fn is_special_dir(_: &Utf8Path) -> bool {
    false
}
