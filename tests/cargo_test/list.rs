use std::{
    ops::Sub,
    path::PathBuf,
    process::Command,
    time::{self, SystemTime},
};

use anyhow::Result;
use assert_cmd::prelude::CommandCargoExt;
use assert_fs::{fixture::PathChild, TempDir};
use fs_set_times::set_mtime;
use walkdir::WalkDir;

use crate::cargo_test::cargo_init;

use super::cargo_build;

#[test]
fn find_new_project_without_git() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let output = Command::cargo_bin("makeclean")?
        .args(["list", "--paths-only", "--min-age", "0", "--any-status"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);

    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;
    let output = PathBuf::try_from(&output.trim())?;
    assert_eq!(&output, project_dir.path());

    Ok(())
}

#[test]
fn finds_project_with_project_type_filter() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    for filter in ["cargo", "rust", "rs"] {
        let output = Command::cargo_bin("makeclean")?
            .args([
                "list",
                "--paths-only",
                "--min-age",
                "0",
                "--any-status",
                "--type",
                filter,
            ])
            .current_dir(&root)
            .output()?;
        dbg!(String::from_utf8(output.stderr)?);

        assert!(output.status.success());
        let output = String::from_utf8(output.stdout)?;
        let output = PathBuf::try_from(&output.trim())?;
        assert_eq!(&output, project_dir.path());
    }

    Ok(())
}

#[test]
fn doesnt_find_project_with_different_project_type_filter() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let output = Command::cargo_bin("makeclean")?
        .args([
            "list",
            "--paths-only",
            "--min-age",
            "0",
            "--any-status",
            "--type",
            "npm",
        ])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);

    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;
    assert!(output.trim().is_empty());

    Ok(())
}

#[test]
fn doesnt_find_new_project_with_default_min_age() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let output = Command::cargo_bin("makeclean")?
        .args(["list", "--paths-only", "--any-status"])
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
        .args(["list", "--paths-only", "--any-status", "--min-age", "4m"])
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
        .args(["list", "--paths-only", "--any-status", "--min-age", "2m"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);

    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;
    let output = PathBuf::try_from(&output.trim())?;
    assert_eq!(&output, project_dir.path());

    Ok(())
}

#[test]
fn by_default_finds_only_projects_than_need_cleaning() -> Result<()> {
    let root = TempDir::new()?;

    let clean_project = {
        let dir = root.child("clean project");
        cargo_init(&dir)?;
        dir
    };

    let built_project = {
        let dir = root.child("built project");
        cargo_init(&dir)?;
        cargo_build(&dir)?;
        dir
    };

    // Using `--any-status` we see both projects:

    let output = Command::cargo_bin("makeclean")?
        .args(["list", "--paths-only", "--min-age", "0", "--any-status"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);

    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;
    let output: Vec<&str> = output.trim().split('\n').collect();
    assert_eq!(output.len(), 2);
    let paths: Vec<PathBuf> = output
        .iter()
        .map(|line| PathBuf::try_from(&line).unwrap())
        .collect();
    assert!(paths.iter().any(|path| path == clean_project.path()));
    assert!(paths.iter().any(|path| path == built_project.path()));

    // But with the default (no `--any-status`), only the built project is listed:

    let output = Command::cargo_bin("makeclean")?
        .args(["list", "--paths-only", "--min-age", "0"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);

    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;
    let output: Vec<&str> = output.trim().split('\n').collect();
    assert_eq!(output.len(), 1);
    let path = PathBuf::try_from(output[0]).unwrap();
    assert_eq!(path, built_project.path());

    Ok(())
}
