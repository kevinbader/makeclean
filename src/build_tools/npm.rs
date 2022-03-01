use super::{BuildStatus, BuildTool, BuildToolProbe};
use crate::{build_tool_manager::BuildToolManager, fs::dir_size};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(NpmProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct NpmProbe;

impl BuildToolProbe for NpmProbe {
    fn probe(&self, path: &Utf8Path) -> Option<Box<dyn BuildTool>> {
        if path.join("package.json").is_file() {
            Some(Box::new(Npm {
                path: path.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, name: &str) -> bool {
        // `name` should already be lowercase, but let's be defensive
        let name = name.to_lowercase();
        name == "npm"
    }
}

#[derive(Debug)]
pub struct Npm {
    path: Utf8PathBuf,
}

impl BuildTool for Npm {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        let deps_dir = self.path.join("node_modules");
        if deps_dir.exists() {
            assert!(deps_dir.is_dir());
            if dry_run {
                println!("{}: rm -r {deps_dir}", self.path);
            } else {
                fs::remove_dir_all(deps_dir)?;
            }
        }

        // TODO: also delete build directory, depending on the language(s) used

        Ok(())
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        let deps_dir = self.path.join("node_modules");
        let status = if deps_dir.exists() {
            let freeable_bytes = dir_size(deps_dir.as_ref());
            BuildStatus::Built { freeable_bytes }
        } else {
            BuildStatus::Clean
        };
        Ok(status)
    }
}

impl std::fmt::Display for Npm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NPM")
    }
}
