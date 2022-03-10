use std::path::Path;

use crate::{build_tool_manager::BuildToolManager, project::ProjectFilter, Project};

use ignore::WalkBuilder;
use tracing::warn;

/// An iterator over [`Project`]s in and below a given directory.
pub fn projects_below<'a>(
    path: &Path,
    project_filter: &'a ProjectFilter,
    build_tool_manager: &'a BuildToolManager,
) -> impl Iterator<Item = Project> + 'a {
    let path = path.canonicalize().expect("canonicalized path");

    WalkBuilder::new(&path)
        .standard_filters(true)
        // skip ignored directories even outside Git repositories
        .require_git(false)
        .build()
        // ignore any errors
        .filter_map(|result| result.ok())
        .filter_map(|entry| entry.path().canonicalize().ok())
        .filter_map(
            |path| match Project::from_dir(&path, project_filter, build_tool_manager) {
                Ok(maybe_project) => maybe_project,
                Err(e) => {
                    warn!("Failed to parse project at {}: {e}", path.display());
                    None
                }
            },
        )
}

#[cfg(test)]
mod test {
    use std::{fmt::Display, fs::OpenOptions, path::Path};

    use assert_fs::{
        fixture::{FileWriteStr, PathChild, PathCreateDir},
        TempDir,
    };
    use chrono::Duration;

    use crate::{
        build_tool_manager::BuildToolManager,
        build_tools::{BuildTool, BuildToolProbe},
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
        fn probe(&self, path: &Path) -> Option<Box<dyn BuildTool>> {
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
        let path = root.path().canonicalize().unwrap();

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
        let root_path = root.path().canonicalize().unwrap();

        let subdir = root.child("subdir");
        subdir.create_dir_all().unwrap();
        let subdir_path = subdir.path().canonicalize().unwrap();

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
        let root_path = root.path().canonicalize().unwrap();

        let subdir = root.child("subdir");
        subdir.create_dir_all().unwrap();
        let subdir_path = subdir.path().canonicalize().unwrap();

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
        let root_path = root.path().canonicalize().unwrap();

        let hidden_dir = root.child(".hidden-dir");
        hidden_dir.create_dir_all().unwrap();
        let hidden_dir_path = hidden_dir.path().canonicalize().unwrap();

        fake_project_at(&hidden_dir_path);
        let projects: Vec<Project> =
            projects_below(&root_path, &project_filter(), &build_tool_manager()).collect();

        dbg!(&projects);
        assert!(projects.is_empty());
    }

    #[test]
    fn skips_project_in_gitignored_dir_even_outside_git_repositories() {
        let root = TempDir::new().unwrap();
        let root_path = root.path().canonicalize().unwrap();

        // write .gitignore file but don't initialize a repository
        root.child(".gitignore").write_str("/ignored-dir/").unwrap();

        let ignored_dir = root.child("ignored-dir");
        ignored_dir.create_dir_all().unwrap();
        let ignored_dir_path = ignored_dir.path().canonicalize().unwrap();

        fake_project_at(&ignored_dir_path);
        let projects: Vec<Project> =
            projects_below(&root_path, &project_filter(), &build_tool_manager()).collect();

        dbg!(&projects);
        assert!(projects.is_empty());
    }
}
