use anyhow::Result;
use assert_cmd::prelude::CommandCargoExt;
use assert_fs::{
    fixture::{FileTouch, PathChild},
    TempDir,
};
use std::{
    fs::{self, File},
    process::Command,
};
use xz::read::XzDecoder;

use crate::util::cargo::{cargo_build, cargo_init};

#[test]
fn archive_cleans_then_packs_includes_hidden_files_then_removes_project_files() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("project");
    cargo_init(&project_dir)?;
    cargo_build(&project_dir)?;

    // Just for fun, also add a hidden file and later make sure it's still there
    project_dir.child(".hidden-test").touch()?;

    let target_dir = project_dir.child("target");
    assert!(target_dir.path().exists());

    let output = Command::cargo_bin("makeclean")?
        .args(["--archive", "--min-age", "0", "--type", "cargo", "--yes"])
        .current_dir(&root)
        .output()?;

    // It runs successfully:
    assert_eq!(String::from_utf8(output.stderr)?.trim(), "");
    assert!(output.status.success());

    // The only thing remaining is the tar file (filename is project name)

    let fname = "cargo_test_project.tar.xz";
    let files_present: Vec<String> = fs::read_dir(&project_dir)
        .unwrap()
        .map(|x| x.unwrap())
        .map(|x| x.file_name().into_string().unwrap())
        .collect();
    assert_eq!(
        files_present.len(),
        1,
        "Expected only the tar file, got: {files_present:?}"
    );
    assert_eq!(files_present[0], fname);

    // The zip doesn't contain the build dir:

    // Let's use a new temporary directory for this..
    let extract_root = TempDir::new()?;

    let tar_xz = File::open(&project_dir.join(fname)).unwrap();
    let tar = XzDecoder::new(tar_xz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(extract_root.path()).unwrap();

    // Cargo.toml was extracted:
    assert!(extract_root.child("Cargo.toml").exists());
    // The `target` dir doesn't:
    assert!(!extract_root.child("target").exists());
    // Finally, the hidden file was also included in the zip file:
    assert!(extract_root.child(".hidden-test").exists());

    Ok(())
}
