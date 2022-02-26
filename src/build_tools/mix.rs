use super::{BuildStatus, BuildTool, BuildToolProbe};
use crate::{fs::dir_size, BuildToolManager};
use anyhow::{bail, Context};
use camino::{Utf8Path, Utf8PathBuf};
use std::process::Command;

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(MixProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub(crate) struct MixProbe;

impl BuildToolProbe for MixProbe {
    fn probe(&self, path: &Utf8Path) -> Option<Box<dyn BuildTool>> {
        if path.join("mix.exs").is_file() {
            Some(Box::new(Mix {
                path: path.to_owned(),
            }))
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
pub(crate) struct Mix {
    path: Utf8PathBuf,
}

impl BuildTool for Mix {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        let mut cmd = Command::new("mix");
        let cmd = cmd.args(["clean", "--deps"]).current_dir(&self.path);
        if dry_run {
            println!("{}: {:?}", self.path, cmd);
        } else {
            let status = cmd.status().with_context(|| {
                format!("Failed to execute {:?} for project at {}", cmd, self.path)
            })?;
            if !status.success() {
                bail!(
                    "Unexpected exit code {} for {:?} for project at {}",
                    status,
                    cmd,
                    self.path
                );
            }
        }
        Ok(())
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        let build_dir = self.path.join("_build");
        let deps_dir = self.path.join("deps");
        let status = if build_dir.exists() || deps_dir.exists() {
            let freeable_bytes = dir_size(build_dir.as_ref()) + dir_size(deps_dir.as_ref());
            BuildStatus::Built { freeable_bytes }
        } else {
            BuildStatus::Clean
        };
        Ok(status)
    }
}

impl std::fmt::Display for Mix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mix")
    }
}
