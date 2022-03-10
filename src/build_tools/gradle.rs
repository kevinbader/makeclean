use crate::build_tool_manager::BuildToolManager;

use super::{BuildTool, BuildToolProbe};
use anyhow::{bail, Context};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(GradleProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct GradleProbe;

impl BuildToolProbe for GradleProbe {
    fn probe(&self, path: &Path) -> Option<Box<dyn BuildTool>> {
        if path.join("build.gradle").is_file() {
            Some(Box::new(Gradle {
                path: path.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, name: &str) -> bool {
        // `name` should already be lowercase, but let's be defensive
        let name = name.to_lowercase();
        name == "gradle"
    }
}

#[derive(Debug)]
pub struct Gradle {
    path: PathBuf,
}

impl BuildTool for Gradle {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        let mut cmd = Command::new("gradle");
        let cmd = cmd.arg("clean").current_dir(&self.path);
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
}

impl std::fmt::Display for Gradle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gradle")
    }
}
