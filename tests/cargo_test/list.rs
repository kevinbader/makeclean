use std::{
    ops::Sub,
    process::Command,
    time::{self, SystemTime},
};

use anyhow::Result;
use assert_cmd::prelude::CommandCargoExt;
use assert_fs::{fixture::PathChild, TempDir};
use fs_set_times::set_mtime;
use makeclean::ProjectDto;
use walkdir::WalkDir;

use crate::cargo_test::cargo_init;

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
    assert_eq!(project.path, project_dir.path().to_str().unwrap());

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
        assert_eq!(project.path, project_dir.path().to_str().unwrap());
    }

    Ok(())
}

#[test]
fn doesnt_find_project_with_different_project_type_filter() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--type", "npm"])
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
fn doesnt_find_new_project_with_min_age_set() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--min-age", "1d"])
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
fn finds_projects_according_to_min_age() -> Result<()> {
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

    // We don't find the project with --min-age=4m

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--json", "--min-age", "4m"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    assert!(
        output.trim().is_empty(),
        "Expected no output, got: {output:?}"
    );

    // However, we find the project with --min-age=2m

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--json", "--min-age", "2m"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    // We expect a single line/project
    let project: ProjectDto = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(project.path, project_dir.path().to_str().unwrap());

    Ok(())
}
