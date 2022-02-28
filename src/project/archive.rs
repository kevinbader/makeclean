use anyhow::{bail, Context, Result};
use assert_fs::{fixture::PathChild, TempDir};
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    fs::{self, File},
    path::Path,
};


use xz::write::XzEncoder;


use crate::{fs::canonicalized, Project};

impl Project {
    /// Move the project's files into an archive.
    pub fn archive(&mut self, dry_run: bool) -> Result<Utf8PathBuf> {
        // The archive is created in a temporary directory. On success, the
        // project directory is renamed, then the archive is moved to the
        // project's original location, then the renamed project directory is
        // removed.

        let tar_xz_fname = format!("{}.tar.xz", self.name);
        let final_tar_xz_path = self.path.join(&tar_xz_fname);

        if final_tar_xz_path.exists() {
            bail!(
                "Cannot create archive at {:?}: there's already a file at that path",
                final_tar_xz_path
            );
        }

        if dry_run {
            println!(
                "Would replace '{}/*' with {:?}",
                self.path, final_tar_xz_path
            );
            return Ok(final_tar_xz_path);
        }

        // Create the archive in a temporary directory
        let tempdir = TempDir::new()?;
        let temp_tar_xz = tempdir.child(&tar_xz_fname);
        create_tar_xz(self.path.as_std_path(), temp_tar_xz.path())?;

        // Rename the project directory
        let renamed_project_path = rename_project_dir(&self.path)?;

        // Copy the archive to its final location
        fs::create_dir(&self.path)
            .with_context(|| format!("Failed to create target directory at {:?}", self.path))?;
        fs::copy(temp_tar_xz.path(), final_tar_xz_path.as_std_path()).with_context(|| {
            format!(
                "Failed to copy {:?} to {:?}",
                temp_tar_xz.path(),
                final_tar_xz_path
            )
        })?;
        tempdir.close()?;

        // Remove the project's contents
        fs::remove_dir_all(renamed_project_path.as_std_path())?;

        Ok(final_tar_xz_path)
    }
}

fn create_tar_xz(src_dir: &Path, dst_path: &Path) -> anyhow::Result<()> {
    let tar_xz = File::create(dst_path)?;
    let xz = XzEncoder::new(tar_xz, 6);
    let mut tar = tar::Builder::new(xz);
    tar.append_dir_all(".", src_dir)?;
    let xz = tar.into_inner()?;
    xz.finish()?;
    Ok(())
}

fn rename_project_dir(project_path: &Utf8Path) -> anyhow::Result<Utf8PathBuf> {
    let project_path = canonicalized(project_path)?;

    let parent = match project_path.parent() {
        Some(dir) => dir,
        None => bail!("No parent directory for project at {:?}", project_path),
    };

    let project_dir_name = project_path
        .file_name()
        .expect("canonicalized never ends with '.' or '..'");

    // Find an available name
    let mut new_path = None;
    for i in 1..10 {
        let candidate = parent.join(format!(".{project_dir_name}~{i}"));
        if !candidate.exists() {
            new_path = Some(candidate);
            break;
        }
    }

    if let Some(new_path) = new_path {
        fs::rename(project_path.as_std_path(), new_path.as_std_path())
            .with_context(|| format!("Failed to rename {:?} to {:?}", project_path, new_path))?;
        Ok(new_path)
    } else {
        bail!("Could not move the project directory after archiving it. Please make sure there are no '.{}~*' directories at {}", project_dir_name, parent)
    }
}
