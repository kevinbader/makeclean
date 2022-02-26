use anyhow::Result;
use assert_cmd::prelude::CommandCargoExt;
use assert_fs::{
    fixture::{FileTouch, PathChild},
    TempDir,
};
use std::{
    fs::{self, OpenOptions},
    process::Command,
};
use zip::ZipArchive;

use crate::cargo_test::cargo_init;

use super::cargo_build;

#[test]
fn clean_built_project_removes_target_dir() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;
    cargo_build(&project_dir)?;

    let target_dir = project_dir.child("target");
    assert!(target_dir.path().exists());

    let output = Command::cargo_bin("makeclean")?
        .args(["clean", "--min-age", "0", "--type", "cargo", "--yes"])
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
fn cleaning_with_dry_run_is_a_noop() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;
    cargo_build(&project_dir)?;

    let target_dir = project_dir.child("target");
    assert!(target_dir.path().exists());

    let output = Command::cargo_bin("makeclean")?
        .args([
            "clean",
            "--min-age",
            "0",
            "--type",
            "cargo",
            "--yes",
            "--dry-run",
        ])
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
        .args(["clean", "--min-age", "0", "--type", "cargo", "--yes"])
        .current_dir(&root)
        .output()?;

    // It works:

    assert_eq!(String::from_utf8(output.stderr)?.trim(), "");
    assert!(output.status.success());

    // TODO: Assert that indeed nothing has changed here?

    Ok(())
}

#[test]
fn passing_zip_to_clean_cleans_the_project_first() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;
    cargo_build(&project_dir)?;

    // Just for fun, also add a hidden file and later make sure it's still there
    project_dir.child(".hidden-test").touch()?;

    let target_dir = project_dir.child("target");
    assert!(target_dir.path().exists());

    let output = Command::cargo_bin("makeclean")?
        .args([
            "clean",
            "--min-age",
            "0",
            "--type",
            "cargo",
            "--yes",
            "--zip",
        ])
        .current_dir(&root)
        .output()?;

    // It runs successfully:
    assert_eq!(String::from_utf8(output.stderr)?.trim(), "");
    assert!(output.status.success());

    // The only thing remaining is the zip file (filename is project name)

    let zip_fname = "cargo_test_project.zip";
    let files_present: Vec<String> = fs::read_dir(&project_dir)
        .unwrap()
        .map(|x| x.unwrap())
        .map(|x| x.file_name().into_string().unwrap())
        .collect();
    assert_eq!(
        files_present.len(),
        1,
        "Expected only the zip file, got: {files_present:?}"
    );
    assert_eq!(files_present[0], zip_fname);

    // The zip doesn't contain the build dir:

    let zip_file = OpenOptions::new()
        .read(true)
        .open(project_dir.join(zip_fname))
        .unwrap();
    let mut zip = ZipArchive::new(zip_file).unwrap();
    // Let's use a new temporary directory for this..
    let extract_root = TempDir::new()?;
    zip.extract(&extract_root).unwrap();
    // Cargo.toml was extracted:
    assert!(extract_root.child("Cargo.toml").exists());
    // The `target` dir doesn't:
    assert!(!extract_root.child("target").exists());
    // Finally, the hidden file was also included in the zip file:
    assert!(extract_root.child(".hidden-test").exists());

    Ok(())
}
