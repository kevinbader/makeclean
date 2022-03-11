use super::{BuildStatus, BuildTool, BuildToolProbe};
use crate::{
    build_tool_manager::BuildToolManager,
    fs::{dir_size, is_gitignored},
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::debug;

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(MixProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct MixProbe;

impl BuildToolProbe for MixProbe {
    fn probe(&self, path: &Path) -> Option<Box<dyn BuildTool>> {
        if path.join("mix.exs").is_file() {
            Some(Box::new(Mix::new(path)))
        } else {
            None
        }
    }

    fn applies_to(&self, name: &str) -> bool {
        // `name` should already be lowercase, but let's be defensive
        let name = name.to_lowercase();
        ["mix", "elixir", "ex", "exs"].contains(&name.as_str())
    }
}

#[derive(Debug)]
pub struct Mix {
    path: PathBuf,
}

impl Mix {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
        }
    }

    fn dir(&self, name: &str) -> Option<PathBuf> {
        let dir = self.path.join(name);
        if dir.is_dir() {
            // Directories are only considered if they are ignored by Git, as
            // `mix clean` should be good enough and removing any other
            // directories is a nice to have.
            if is_gitignored(&self.path, &dir) {
                Some(dir)
            } else {
                debug!(
                    "Skipping directory as not ignored by Git: {}",
                    dir.display()
                );
                None
            }
        } else {
            None
        }
    }
}

static EPHEMERAL_DIRS: &[&str] = &["_build", "deps", ".elixir_ls"];

impl BuildTool for Mix {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        // `mix clean --deps` exists, but
        // - it needs to be installed
        // - it needs to match the version used in the project
        // - it doesn't remove everything, so projects are still reported un-cleaned
        //
        // So instead, we're simply deleting the well-known directories, which
        // works just as well (better?), is faster, and doesn't require mix to
        // be installed.

        for dir in EPHEMERAL_DIRS.iter().filter_map(|x| self.dir(x)) {
            if dry_run {
                println!("rm -r '{}'", dir.display());
            } else {
                fs::remove_dir_all(dir)?;
            }
        }

        Ok(())
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        let size: u64 = EPHEMERAL_DIRS
            .iter()
            .filter_map(|x| self.dir(x))
            .map(|dir| dir_size(&dir))
            .sum();

        let status = match size {
            0 => BuildStatus::Clean,
            freeable_bytes => BuildStatus::Built { freeable_bytes },
        };

        Ok(status)
    }

    fn project_name(&self) -> Option<anyhow::Result<String>> {
        // mix.exs, which contains the project name, is not easy to parse without Elixir.
        // While `mix run -e 'IO.puts(Mix.Project.config[:app])'` would work, it would
        // also compile the application, which is of course an unintended side effect.
        // To prevent false positives, we don't even try.
        None
    }
}

impl std::fmt::Display for Mix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mix")
    }
}

#[cfg(test)]
mod test {
    use assert_fs::{
        fixture::{FileWriteStr, PathChild},
        TempDir,
    };

    use super::*;

    #[test]
    fn elixir_ls_cache_is_only_removed_if_gitignored() {
        let root = TempDir::new().unwrap();

        root.child("normal")
            .child(".elixir_ls")
            .child("dummy")
            .write_str("dummy")
            .unwrap();
        root.child("normal")
            .child(".gitignore")
            .write_str(".elixir_ls/")
            .unwrap();

        root.child("not-ignored")
            .child(".elixir_ls")
            .child("dummy")
            .write_str("dummy")
            .unwrap();

        // In the normal case, status should report back that the project is not clean:
        let normal_status = Mix::new(&root.child("normal")).status().unwrap();
        assert!(matches!(normal_status, BuildStatus::Built{freeable_bytes} if freeable_bytes > 0));

        // If not ignored, however, the directory is not considered, so the project is clean:
        let not_ignored_status = Mix::new(&root.child("not-ignored")).status().unwrap();
        assert!(matches!(not_ignored_status, BuildStatus::Clean));
    }
}
