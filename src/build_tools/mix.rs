use super::{BuildStatus, BuildTool, BuildToolProbe};
use crate::{build_tool_manager::BuildToolManager, fs::dir_size};
use anyhow::{bail, Context};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub fn register(manager: &mut BuildToolManager, probe_only: bool) -> anyhow::Result<()> {
    if !probe_only {
        let mix_is_installed = Command::new("mix")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !mix_is_installed {
            bail!("mix is not available");
        }
    }

    let probe = Box::new(MixProbe {});
    manager.register(probe);

    Ok(())
}

#[derive(Debug)]
pub struct MixProbe;

impl BuildToolProbe for MixProbe {
    fn probe(&self, path: &Path) -> Option<Box<dyn BuildTool>> {
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
pub struct Mix {
    path: PathBuf,
}

impl BuildTool for Mix {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        let mut cmd = Command::new("mix");
        let cmd = cmd.args(["clean", "--deps"]).current_dir(&self.path);
        if dry_run {
            println!("{}: {:?}", self.path.display(), cmd);
        } else {
            let status = cmd.status().with_context(|| {
                format!(
                    "Failed to execute {:?} for project at {}",
                    cmd,
                    self.path.display()
                )
            })?;
            if !status.success() {
                bail!(
                    "Unexpected exit code {} for {:?} for project at {}",
                    status,
                    cmd,
                    self.path.display()
                );
            }
        }
        Ok(())
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        let build_dir = self.path.join("_build");
        let deps_dir = self.path.join("deps");
        let status = if build_dir.exists() || deps_dir.exists() {
            match dir_size(build_dir.as_ref()) + dir_size(deps_dir.as_ref()) {
                0 => BuildStatus::Clean,
                freeable_bytes => BuildStatus::Built { freeable_bytes },
            }
        } else {
            BuildStatus::Clean
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
