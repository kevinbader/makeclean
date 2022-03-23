use crate::build_tool_manager::BuildToolManager;

use super::{BuildTool, BuildToolKind, BuildToolProbe};
use anyhow::{bail, Context};
use displaydoc::Display;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(MavenProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct MavenProbe;

impl BuildToolProbe for MavenProbe {
    fn probe(&self, dir: &Path) -> Option<Box<dyn BuildTool>> {
        if dir.join("pom.xml").is_file() {
            Some(Box::new(Maven {
                dir: dir.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, kind: BuildToolKind) -> bool {
        use BuildToolKind::*;
        matches!(kind, Maven | Mvn)
    }
}

#[derive(Debug, Display)]
/// Maven
pub struct Maven {
    dir: PathBuf,
}

impl BuildTool for Maven {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        let mut cmd = Command::new("mvn");
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
