use crate::build_tool_manager::BuildToolManager;

use super::{BuildTool, BuildToolProbe};
use anyhow::{bail, Context};
use camino::{Utf8Path, Utf8PathBuf};
use std::process::Command;

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(MavenProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct MavenProbe;

impl BuildToolProbe for MavenProbe {
    fn probe(&self, path: &Utf8Path) -> Option<Box<dyn BuildTool>> {
        if path.join("pom.xml").is_file() {
            Some(Box::new(Maven {
                path: path.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, name: &str) -> bool {
        // `name` should already be lowercase, but let's be defensive
        let name = name.to_lowercase();
        ["maven", "mvn"].contains(&name.as_str())
    }
}

#[derive(Debug)]
pub struct Maven {
    path: Utf8PathBuf,
}

impl BuildTool for Maven {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        let mut cmd = Command::new("mvn");
        let cmd = cmd.arg("clean").current_dir(&self.path);
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
}

impl std::fmt::Display for Maven {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Maven")
    }
}
