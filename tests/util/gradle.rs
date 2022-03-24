use std::{fs, io, path::Path, process::Command};

use anyhow::{bail, Result};
use assert_fs::fixture::{FileWriteStr, PathChild};
use tracing::warn;

pub fn gradle_init<T>(parent: &T) -> Result<()>
where
    T: PathChild + AsRef<Path>,
{
    fs::create_dir_all(parent.as_ref())?;
    match Command::new("gradle")
        .args([
            "init",
            "--project-name",
            "gradle_test_project",
            "--type",
            "basic",
        ])
        .current_dir(parent.as_ref())
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => bail!("gradle init failed: {:?}", output),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            warn!("failed to exec gradle: {}", e);
            // not installed on this system.. let's fake it then
            let settings_gradle = r#"
                rootProject.name = 'gradle_test_project'
                "#;
            parent.child("settings.gradle").write_str(settings_gradle)?;
            parent.child("build.gradle").write_str("")?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
