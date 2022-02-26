use anyhow::{bail, format_err, Context, Result};
use assert_fs::{fixture::PathChild, TempDir};
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
};
use tracing::{debug, trace};
use walkdir::WalkDir;
use xz::write::XzEncoder;
use zip::{write::FileOptions, ZipWriter};

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

    #[deprecated]
    pub fn zip(&mut self, dry_run: bool) -> Result<Utf8PathBuf> {
        let base_path = &self.path;
        let zip_fname = format!("{}.zip", self.name);

        let zip_path = base_path.join(&zip_fname);
        if zip_path.exists() {
            bail!(
                "Cannot create zip archive at {:?}: there's already a file at that path",
                zip_path
            );
        }

        let mut project_toplevel_files: Vec<Utf8PathBuf> = vec![];
        for entry in fs::read_dir(&self.path)
            .with_context(|| "Could not read project dir while zip'ing it")?
        {
            let entry = entry?;
            project_toplevel_files.push(entry.path().try_into()?);
        }

        if dry_run {
            println!(
                "Would zip {base_path:?} as {zip_fname:?}, then delete {project_toplevel_files:?}"
            );
            return Ok(zip_path);
        }

        let zip_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(zip_path.as_std_path())
            .with_context(|| format!("Failed to open new zip file at {zip_path:?}"))?;
        let mut zip = ZipWriter::new(zip_file);
        let options = FileOptions::default();

        for entry in WalkDir::new(base_path).into_iter() {
            // Abort creating the zip file if there are file read errors
            let entry = entry.with_context(|| "Failed to read file when adding to ZIP archive")?;
            let entry_path = entry.path();

            if *entry_path == zip_path {
                continue;
            }

            let name = entry_path
                .strip_prefix(base_path)?
                .to_str()
                .ok_or_else(|| {
                    format_err!(
                        "Failed to strip base_path {base_path:?} from entry_path {entry_path:?}"
                    )
                })?;

            if entry_path.is_file() {
                trace!("adding file {entry_path:?} as {name:?}");
                zip.start_file(name, options).with_context(|| {
                    format!("While creating the ZIP file, could not start file {name:?}")
                })?;
                let mut f = OpenOptions::new()
                    .read(true)
                    .open(entry_path)
                    .with_context(|| format!("Could not add (open) {entry_path:?} to ZIP file"))?;
                io::copy(&mut f, &mut zip)
                    .with_context(|| format!("Could not add (copy) {entry_path:?} to ZIP file"))?;
            } else if !name.is_empty() {
                // Only if not root! Avoids path spec / warning
                // and mapname conversion failed error on unzip
                trace!("adding dir {entry_path:?} as {name:?} ...");
                zip.add_directory(name, options)
                    .with_context(|| format!("Could not add directory {name:?} to ZIP file"))?;
            }
        }

        zip.finish()?;

        for path in project_toplevel_files {
            debug!("Removing {path:?}");
            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .with_context(|| format!("Could not remove directory at {path:?} while zip'ing - zip file is there, but the project may be half deleted (!)"))?;
            } else {
                fs::remove_file(&path)
                    .with_context(|| format!("Could not remove file at {path:?} while zip'ing - zip file is there, but the project may be half deleted (!)"))?;
            }
        }

        Ok(zip_path)
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
