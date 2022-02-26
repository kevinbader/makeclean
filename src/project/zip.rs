use anyhow::{bail, format_err, Context, Result};
use camino::Utf8PathBuf;
use std::{
    fs::{self, OpenOptions},
    io,
};
use tracing::{debug, trace};
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

use crate::Project;

impl Project {
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
