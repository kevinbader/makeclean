pub mod archive;
pub mod clean;
pub mod list;

use std::{fs, io, path::Path, process::Command};

use anyhow::{bail, Result};
use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild};
use tracing::warn;

pub fn cargo_init<T>(parent: &T) -> Result<()>
where
    T: PathChild + AsRef<Path>,
{
    fs::create_dir_all(parent.as_ref())?;
    match Command::new("cargo")
        .args(["init", "--name", "cargo_test_project", "--vcs", "none", "."])
        .current_dir(parent.as_ref())
        .status()
    {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => bail!("unexpected exit code {:?}", status.code()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            warn!("failed to exec cargo: {}", e);
            // not installed on this system.. let's fake it then
            parent.child("Cargo.toml").touch()?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn cargo_build<T>(parent: &T) -> Result<()>
where
    T: PathChild + AsRef<Path>,
{
    assert!(parent.as_ref().exists());
    match Command::new("cargo")
        .arg("build")
        .current_dir(parent.as_ref())
        .status()
    {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => bail!("unexpected exit code {:?}", status.code()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            warn!("failed to exec cargo: {}", e);
            // not installed on this system.. let's fake it then
            parent
                .child("Cargo.lock")
                .write_str("dependency versions")?;
            parent
                .child("target")
                .child("CACHEDIR.TAG")
                .write_str("caching stuff")?;
            parent
                .child("target")
                .child("debug")
                .child("cargo_test_project")
                .write_str("binary data")?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
