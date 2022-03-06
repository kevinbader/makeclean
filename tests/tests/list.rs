use std::{
    ops::Sub,
    process::Command,
    time::{self, SystemTime},
};

use anyhow::Result;
use assert_cmd::prelude::CommandCargoExt;
use assert_fs::{fixture::PathChild, TempDir};
use fs_set_times::set_mtime;
use makeclean::project::dto::ProjectDto;
use walkdir::WalkDir;

use crate::util::{cargo::cargo_init, fs::canonicalized_str, git::git_init, npm::npm_init};

#[test]
fn find_new_project_without_git() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--json"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    // We expect a single line/project
    let project: ProjectDto = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(project.path, canonicalized_str(&project_dir));

    Ok(())
}

#[test]
fn directories_ignored_by_git_are_not_considered() -> Result<()> {
    // Set up the test directory, with a project in each directory - only one will be found
    let root = TempDir::new()?;
    cargo_init(&root.child("normal_dir"))?;
    cargo_init(&root.child("ignored_dir"))?;
    git_init(&root, "/ignored_dir", true);

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--json"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    // Only the project in `normal_dir` is returned:
    let project: ProjectDto = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(project.path, canonicalized_str(&root.join("normal_dir")));

    Ok(())
}

#[test]
fn subprojects_are_discovered() -> Result<()> {
    // Setup: a Cargo project that contains a NPM project (e.g. a frontend) in a subdirectory.
    let root = TempDir::new()?;
    cargo_init(&root)?;
    npm_init(&root.child("web"))?;

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--json"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    // Both projects are discovered:
    let projects: Vec<ProjectDto> = output
        .trim()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        projects.len(),
        2,
        "Expected both projects, got: {projects:?}"
    );
    assert!(projects
        .iter()
        .any(|p| p.path == canonicalized_str(&root.path()) && p.build_tools == vec!["Cargo"]));
    assert!(projects
        .iter()
        .any(|p| p.path == canonicalized_str(&root.join("web")) && p.build_tools == vec!["NPM"]));

    Ok(())
}

#[test]
fn finds_project_with_project_type_filter() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    for filter in ["cargo", "rust", "rs"] {
        let output = Command::cargo_bin("makeclean")?
            .args(["--list", "--json", "--type", filter])
            .current_dir(&root)
            .output()?;
        dbg!(String::from_utf8(output.stderr)?);
        assert!(output.status.success());
        let output = String::from_utf8(output.stdout)?;

        // We expect a single line/project
        let project: ProjectDto = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(project.path, canonicalized_str(&project_dir));
    }

    Ok(())
}

#[test]
fn doesnt_find_project_with_different_project_type_filter() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let output = Command::cargo_bin("makeclean")?
        .args(["--json", "--list", "--type", "npm"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    assert!(
        output.trim().is_empty(),
        "Expected no output, got: {output:?}"
    );

    Ok(())
}

#[test]
fn doesnt_find_new_project_with_min_stale_set() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let output = Command::cargo_bin("makeclean")?
        .args(["--json", "--list", "--min-stale", "1d"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    assert!(
        output.trim().is_empty(),
        "Expected no output, got: {output:?}"
    );

    Ok(())
}

#[test]
fn finds_projects_according_to_min_stale() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    // Turn back time

    let three_months = time::Duration::from_secs(60 * 60 * 24 * 30 * 3);
    let mtime = SystemTime::now().sub(three_months);
    WalkDir::new(&project_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .for_each(|entry| set_mtime(entry.path(), mtime.into()).unwrap());

    // We don't find the project with --min-stale=4m

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--json", "--min-stale", "4m"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    assert!(
        output.trim().is_empty(),
        "Expected no output, got: {output:?}"
    );

    // However, we find the project with --min-stale=2m

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--json", "--min-stale", "2m"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    // We expect a single line/project
    let project: ProjectDto = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(project.path, canonicalized_str(&project_dir));

    Ok(())
}
