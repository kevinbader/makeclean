use crate::{
    build_tool_manager::BuildToolManager, fs::canonicalized, project::ProjectFilter,
    vcs::VersionControlSystem, Project,
};
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::VecDeque;
use tracing::{debug, trace, warn};

/// An iterator over [`Project`]s in and below a given directory.
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

#[cfg(test)]
mod test {
    use std::{fmt::Display, fs::OpenOptions, path::Path};

    use assert_fs::{
        fixture::{PathChild, PathCreateDir},
        TempDir,
    };
    use camino::Utf8Path;
    use chrono::Duration;

    use crate::{
        build_tool_manager::BuildToolManager,
        build_tools::{BuildTool, BuildToolProbe},
        fs::canonicalized,
        project::{Project, ProjectFilter, StatusFilter},
    };

    use super::projects_below;

    #[derive(Debug)]
    struct TestTool;
    impl BuildTool for TestTool {
        fn clean_project(&mut self, _: bool) -> anyhow::Result<()> {
            unimplemented!("not executed in these tests")
        }
    }
    impl Display for TestTool {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unimplemented!("not executed in these tests")
        }
    }

    #[derive(Debug)]
    struct TestProbe;
    impl BuildToolProbe for TestProbe {
        fn probe(&self, path: &Utf8Path) -> Option<Box<dyn BuildTool>> {
            if path.join("projectfile").exists() {
                Some(Box::new(TestTool {}))
            } else {
                None
            }
        }

        fn applies_to(&self, _: &str) -> bool {
            unimplemented!("not executed in these tests")
        }
    }

    fn fake_project_at(path: impl AsRef<Path>) {
        let _ = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path.as_ref().join("projectfile"))
            .expect("creating the fake project");
    }

    fn build_tool_manager() -> BuildToolManager {
        let mut btm = BuildToolManager::default();
        // remove all default probes
        btm.filter(&[String::new()]);

        let test_probe = Box::new(TestProbe {});
        btm.register(test_probe);

        btm
    }

    fn project_filter() -> ProjectFilter {
        ProjectFilter {
            min_stale: Duration::zero(),
            status: StatusFilter::Any,
        }
    }

    #[test]
    fn finds_project_in_root_dir() {
        let root = TempDir::new().unwrap();
        let path = canonicalized(root.path()).unwrap();

        fake_project_at(&path);
        let projects: Vec<Project> =
            projects_below(&path, &project_filter(), &build_tool_manager()).collect();

        dbg!(&projects);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].path, path);
    }

    #[test]
    fn finds_project_in_subdir() {
        let root = TempDir::new().unwrap();
        let root_path = canonicalized(root.path()).unwrap();

        let subdir = root.child("subdir");
        subdir.create_dir_all().unwrap();
        let subdir_path = canonicalized(subdir.path()).unwrap();

        fake_project_at(&subdir_path);
        let projects: Vec<Project> =
            projects_below(&root_path, &project_filter(), &build_tool_manager()).collect();

        dbg!(&projects);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].path, subdir_path);
    }

    #[test]
    fn finds_nested_project_in_subdir_of_project() {
        let root = TempDir::new().unwrap();
        let root_path = canonicalized(root.path()).unwrap();

        let subdir = root.child("subdir");
        subdir.create_dir_all().unwrap();
        let subdir_path = canonicalized(subdir.path()).unwrap();

        fake_project_at(&root_path);
        fake_project_at(&subdir_path);
        let projects: Vec<Project> =
            projects_below(&root_path, &project_filter(), &build_tool_manager()).collect();

        dbg!(&projects);
        assert_eq!(projects.len(), 2);
        // we expect BFS ordering
        assert_eq!(projects[0].path, root_path);
        assert_eq!(projects[1].path, subdir_path);
    }

    #[test]
    fn skips_project_in_hidden_dir() {
        let root = TempDir::new().unwrap();
        let root_path = canonicalized(root.path()).unwrap();

        let hidden_dir = root.child(".hidden-dir");
        hidden_dir.create_dir_all().unwrap();
        let hidden_dir_path = canonicalized(hidden_dir.path()).unwrap();

        fake_project_at(&hidden_dir_path);
        let projects: Vec<Project> =
            projects_below(&root_path, &project_filter(), &build_tool_manager()).collect();

        dbg!(&projects);
        assert!(projects.is_empty());
    }

    #[test]
    fn skips_project_in_gitignored_dir_if_within_git_repository() {
        let root = TempDir::new().unwrap();
        let root_path = canonicalized(root.path()).unwrap();

        // init a Git repo and write .gitignore file
        git_init(&root, "/ignored-dir/", true);

        let ignored_dir = root.child("ignored-dir");
        ignored_dir.create_dir_all().unwrap();
        let ignored_dir_path = canonicalized(ignored_dir.path()).unwrap();

        fake_project_at(&ignored_dir_path);
        let projects: Vec<Project> =
            projects_below(&root_path, &project_filter(), &build_tool_manager()).collect();

        dbg!(&projects);
        assert!(projects.is_empty());
    }

    fn git_init<T>(parent: &T, gitignore: &str, commit: bool)
    where
        T: PathChild + AsRef<Path>,
    {
        fn git_add(repo: &git2::Repository, pathspecs: &[&str]) -> anyhow::Result<git2::Index> {
            let mut index = repo.index()?;
            index.add_all(pathspecs, git2::IndexAddOption::DEFAULT, None)?;
            index.write()?;

            Ok(index)
        }

        fn git_commit(
            repo: &git2::Repository,
            index: &mut git2::Index,
            message: &str,
        ) -> anyhow::Result<()> {
            let tree_oid = index.write_tree()?;
            let tree = repo.find_tree(tree_oid)?;

            let signature = git2::Signature::now("makeclean-test", "makeclean-test@example.com")?;
            let author = &signature;
            let committer = &signature;

            let parents = match repo.head() {
                Ok(head) => vec![head.peel_to_commit()?],
                Err(e) => {
                    if e.code() == git2::ErrorCode::UnbornBranch {
                        // No commits yet - HEAD does not exist
                        vec![]
                    } else {
                        return Err(e.into());
                    }
                }
            };

            let parents: Vec<&git2::Commit> = parents.iter().collect();

            repo.commit(Some("HEAD"), author, committer, message, &tree, &parents)?;

            Ok(())
        }
        std::fs::create_dir_all(parent.as_ref()).unwrap();
        let repo = git2::Repository::init(&parent).unwrap();

        if !gitignore.is_empty() {
            let gitignore_path = parent.child(".gitignore");
            let mut gitignore_file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(gitignore_path)
                .unwrap();
            std::io::Write::write_all(&mut gitignore_file, gitignore.as_bytes()).unwrap();
        }

        if commit {
            let mut index = git_add(&repo, &["."]).unwrap();
            git_commit(&repo, &mut index, "test").unwrap();
        }
    }
}
