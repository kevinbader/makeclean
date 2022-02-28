use std::{fs, io, path::Path, process::Command};

use anyhow::{bail, Result};
use assert_fs::fixture::{FileTouch, PathChild};
use tracing::warn;

pub fn npm_init<T>(parent: &T) -> Result<()>
where
    T: PathChild + AsRef<Path>,
{
    fs::create_dir_all(parent.as_ref())?;
    match Command::new("npm")
        .args(["init", "--yes"])
        .current_dir(parent.as_ref())
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => bail!("npm init failed: {:?}", output),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            warn!("failed to exec npm: {}", e);
            // not installed on this system.. let's fake it then
            parent.child("package.json").touch()?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
