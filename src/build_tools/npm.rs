use super::{BuildStatus, BuildTool, BuildToolProbe};
use crate::{build_tool_manager::BuildToolManager, fs::dir_size};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(NpmProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct NpmProbe;

impl BuildToolProbe for NpmProbe {
    fn probe(&self, path: &Path) -> Option<Box<dyn BuildTool>> {
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
    path: PathBuf,
}

static NODE_MODULES: &str = "node_modules";

impl Npm {
    fn dir(&self, name: &str) -> Option<PathBuf> {
        let dir = self.path.join(name);
        if dir.is_dir() {
            Some(dir)
        } else {
            None
        }
    }
}

impl BuildTool for Npm {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        if let Some(node_modules) = self.dir(NODE_MODULES) {
            if dry_run {
                println!("{}: rm -r {}", self.path.display(), node_modules.display());
            } else {
                fs::remove_dir_all(node_modules)?;
            }
        }

        // TODO: also delete build directory, depending on the language(s) used

        Ok(())
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        let size: u64 = [NODE_MODULES]
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
}

impl std::fmt::Display for Npm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NPM")
    }
}
