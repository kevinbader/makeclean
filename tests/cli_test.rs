mod tests;
mod util;

use anyhow::Result;
use assert_fs::{prelude::*, TempDir};
use camino::Utf8Path;
use makeclean::{projects_below, BuildToolManager, Project};

use crate::util::{
    cargo::cargo_init, git::git_init, npm::npm_init, project_filter::noop_project_filter,
};

// #[test]
// fn list_finds_projects() -> Result<()> {
//     // We set up a couple of projects and then run `makeclean list` with the
//     // different options, asserting that the filtering works correctly.

//     let root = TempDir::new()?;

//     {
//         let dir = root.child("cargo").child("new_no-git");
//         cargo_init(&dir)?;
//     }

//     {
//         let dir = root.child("cargo").child("new_with-git");
//         cargo_init(&dir)?;
//         git_init(&dir, "", true);
//     }

//     {
//         let dir = root.child("cargo").child("year-old_no-git");
//         cargo_init(&dir)?;
//         let a_year = time::Duration::from_secs(60 * 60 * 24 * 7 * 52);
//         let mtime = SystemTime::now().sub(a_year);
//         WalkDir::new(&dir)
//             .into_iter()
//             .filter_map(|entry| entry.ok())
//             .for_each(|entry| set_mtime(entry.path(), mtime.into()).unwrap());
//     }

//     // TODO year old, with git, change commit time

//     // `makeclean list` finds all projects

//     let output = Command::cargo_bin("makeclean")?
//         .args(["list", "--all"])

//     Ok(())
// }

// #[test]
// fn identifies_projects_by_build_tool() -> Result<()> {
//     type InitFunc = fn(&TempDir) -> Result<()>;
//     let tools: &[(&str, InitFunc)] = &[
//         ("Cargo", cargo_init),
//         ("Elm", elm_init),
//         ("Gradle", gradle_init),
//         ("Maven", maven_init),
//         ("Mix", mix_init),
//         ("NPM", npm_init),
//     ];
//     for (build_tool_name, init) in tools {
//         let root = TempDir::new()?;
//         init(&root)?;

//         let root_path = Utf8Path::from_path(root.path()).unwrap();
//         let projects: Vec<Project> = projects_below(
//             root_path,
//             &noop_project_filter(),
//             &BuildToolManager::default(),
//         )
//         .collect();

//         assert_eq!(projects.len(), 1);
//         assert_eq!(projects[0].path, root_path);
//         assert_eq!(projects[0].build_tools.len(), 1);
//         assert_eq!(projects[0].build_tools[0].to_string(), *build_tool_name);
//     }

//     Ok(())
// }

#[test]
fn directories_ignored_by_git_are_not_considered() -> Result<()> {
    // Set up the test directory, with a project in each directory - only one will be found
    let root = TempDir::new()?;
    cargo_init(&root.child("normal_dir"))?;
    cargo_init(&root.child("ignored_dir"))?;

    git_init(&root, "/ignored_dir", true);

    // Only the project in `normal_dir` is returned:
    let root_path = Utf8Path::from_path(root.path()).unwrap();
    let projects: Vec<Project> = projects_below(
        root_path,
        &noop_project_filter(),
        &BuildToolManager::default(),
    )
    .collect();

    assert_eq!(
        projects.len(),
        1,
        "Expected one projects, got: {projects:?}"
    );
    assert_eq!(projects[0].path, root_path.join("normal_dir"));

    Ok(())
}

#[test]
fn subprojects_are_discovered() -> Result<()> {
    // Setup: a Cargo project that contains a NPM project (e.g. a frontend) in a subdirectory.
    let root = TempDir::new()?;
    cargo_init(&root)?;
    npm_init(&root.child("web"))?;

    // Both projects are discovered:
    let root_path = Utf8Path::from_path(root.path()).unwrap();
    let projects: Vec<Project> = projects_below(
        root_path,
        &noop_project_filter(),
        &BuildToolManager::default(),
    )
    .collect();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].path, root_path);
    assert_eq!(projects[0].build_tools.len(), 1);
    assert_eq!(projects[0].build_tools[0].to_string(), "Cargo");
    assert_eq!(projects[1].path, root_path.join("web"));
    assert_eq!(projects[1].build_tools.len(), 1);
    assert_eq!(projects[1].build_tools[0].to_string(), "NPM");

    Ok(())
}
