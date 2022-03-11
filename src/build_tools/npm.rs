use super::{remove_dirs, status_from_dirs, BuildStatus, BuildTool, BuildToolProbe};
use crate::build_tool_manager::BuildToolManager;
use std::path::{Path, PathBuf};

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

static EPHEMERAL_DIRS: &[&str] = &["node_modules"];

impl BuildTool for Npm {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        // TODO: also delete build directory, depending on the language(s) used
        remove_dirs(&self.path, EPHEMERAL_DIRS, dry_run)
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        status_from_dirs(&self.path, EPHEMERAL_DIRS)
    }
}

impl std::fmt::Display for Npm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NPM")
    }
}
