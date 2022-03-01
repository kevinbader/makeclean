use anyhow::Result;
use assert_cmd::prelude::CommandCargoExt;
use assert_fs::{fixture::PathChild, TempDir};
use std::process::Command;

use crate::util::cargo::{cargo_build, cargo_init};

#[test]
fn the_prompt_only_lists_projects_that_need_cleaning() -> Result<()> {
    let root = TempDir::new()?;

    let _clean_project = {
        let dir = root.child("the-clean-project");
        cargo_init(&dir)?;
        dir
    };

    let _built_project = {
        let dir = root.child("the-built-project");
        cargo_init(&dir)?;
        cargo_build(&dir)?;
        dir
    };

    // Only the built project is listed:

    let output = Command::cargo_bin("makeclean")?
        .args(["--dry-run", "--min-stale", "0", "--json"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    assert!(output.contains("the-built-project"));
    assert!(!output.contains("the-clean-project"));

    Ok(())
}

#[test]
fn doesnt_consider_new_project_with_default_min_stale_setting() -> Result<()> {
    // Note that `--list` has a different default (--min-stale=0)!

    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let output = Command::cargo_bin("makeclean")?
        .args(["--json", "--dry-run"])
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
fn cleaning_a_built_project_removes_target_dir() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;
    cargo_build(&project_dir)?;

    let target_dir = project_dir.child("target");
    assert!(target_dir.path().exists());

    let output = Command::cargo_bin("makeclean")?
        .args(["--min-stale", "0", "--type", "cargo", "--yes"])
        .current_dir(&root)
        .output()?;

    // It runs successfully:
    assert_eq!(String::from_utf8(output.stderr)?.trim(), "");
    assert!(output.status.success());

    // `target/` is now gone:
    assert!(!target_dir.path().exists());

    Ok(())
}

#[test]
fn cleaning_with_yes_and_dry_run_is_a_dry_run() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;
    cargo_build(&project_dir)?;

    let target_dir = project_dir.child("target");
    assert!(target_dir.path().exists());

    let output = Command::cargo_bin("makeclean")?
        .args(["--min-stale", "0", "--type", "cargo", "--yes", "--dry-run"])
        .current_dir(&root)
        .output()?;

    // It runs successfully:
    assert_eq!(String::from_utf8(output.stderr)?.trim(), "");
    assert!(output.status.success());

    // `target/` is still there:
    assert!(target_dir.path().exists());

    Ok(())
}

#[test]
fn cleaning_a_cleaned_project_is_a_noop() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;

    let target_dir = project_dir.child("target");
    assert!(!target_dir.path().exists());

    let output = Command::cargo_bin("makeclean")?
        .args(["--min-stale", "0", "--type", "cargo", "--yes"])
        .current_dir(&root)
        .output()?;

    // It works:

    assert_eq!(String::from_utf8(output.stderr)?.trim(), "");
    assert!(output.status.success());

    // TODO: Assert that indeed nothing has changed here?

    Ok(())
}
