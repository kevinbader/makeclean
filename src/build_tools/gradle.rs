use crate::build_tool_manager::BuildToolManager;

use super::{BuildTool, BuildToolKind, BuildToolProbe};
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
    fn probe(&self, dir: &Path) -> Option<Box<dyn BuildTool>> {
        if dir.join("build.gradle").is_file() {
            Some(Box::new(Gradle {
                dir: dir.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, kind: BuildToolKind) -> bool {
        kind == BuildToolKind::Gradle
    }
}

#[derive(Debug)]
pub struct Gradle {
    dir: PathBuf,
}

impl BuildTool for Gradle {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        let mut cmd = Command::new("gradle");
        let cmd = cmd.arg("clean").current_dir(&self.dir);
        if dry_run {
            println!("{}: {:?}", self.dir.display(), cmd);
        } else {
            let status = cmd.status().with_context(|| {
                format!(
                    "Failed to execute {:?} for project at {}",
                    cmd,
                    self.dir.display()
                )
            })?;
            if !status.success() {
                bail!(
                    "Unexpected exit code {} for {:?} for project at {}",
                    status,
                    cmd,
                    self.dir.display()
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
