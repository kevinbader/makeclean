use std::{fs, io, path::Path, process::Command};

use anyhow::{bail, Result};
use assert_fs::fixture::{FileWriteStr, PathChild};
use tracing::warn;

pub fn flutter_init<T>(parent: &T) -> Result<()>
where
    T: PathChild + AsRef<Path>,
{
    fs::create_dir_all(parent.as_ref())?;
    match Command::new("flutter")
        .args([
            "create",
            "--no-pub",
            "--project-name",
            "flutter_test_project",
            ".",
        ])
        .current_dir(parent.as_ref())
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => bail!("flutter create failed: {:?}", output),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            warn!("failed to exec flutter: {}", e);
            // not installed on this system.. let's fake it then
            let pubspec_yaml = r#"
                name: flutter_test_project
                version: 1.0.0+1
                flutter: {}
                "#;
            parent.child("pubspec.yaml").write_str(pubspec_yaml)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
