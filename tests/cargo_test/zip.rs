use anyhow::Result;
use assert_cmd::prelude::CommandCargoExt;
use assert_fs::{fixture::PathChild, TempDir};
use std::{fs, process::Command};
use zip::ZipArchive;

use crate::cargo_test::cargo_init;

use super::cargo_build;

#[test]
fn zip_does_not_clean_the_project_before_creating_the_zip_file() -> Result<()> {
    todo!()
}

#[test]
fn zip_removes_everything_after_creating_the_zip_file() -> Result<()> {
    todo!();

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
            "--zip",
        ])
        .current_dir(&root)
        .output()?;

    // It runs successfully:
    assert_eq!(String::from_utf8(output.stderr)?.trim(), "");
    assert!(output.status.success());

    // The only thing remaining is the zip file (filename is project name)

    let zip_fname = "cargo_test_project";
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

    let zip_file = fs::File::open(zip_fname).unwrap();
    let mut zip = ZipArchive::new(zip_file).unwrap();
    zip.extract(&project_dir).unwrap();
    // Cargo.toml was extracted:
    assert!(project_dir.child("Cargo.toml").exists());
    // The `target` dir doesn't:
    assert!(!project_dir.child("target").exists());

    Ok(())
}

#[test]
fn zip_considers_hidden_files_when_packing_and_deleting_files() -> Result<()> {
    todo!()
}
