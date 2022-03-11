use super::{remove_dirs, status_from_dirs, BuildStatus, BuildTool, BuildToolProbe};
use crate::{build_tool_manager::BuildToolManager};
use std::{
    path::{Path, PathBuf},
};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(ElmProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct ElmProbe;

impl BuildToolProbe for ElmProbe {
    fn probe(&self, path: &Path) -> Option<Box<dyn BuildTool>> {
        if path.join("elm.json").is_file() {
            Some(Box::new(Elm {
                path: path.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, name: &str) -> bool {
        // `name` should already be lowercase, but let's be defensive
        let name = name.to_lowercase();
        name == "elm"
    }
}

#[derive(Debug)]
pub struct Elm {
    path: PathBuf,
}

static EPHEMERAL_DIRS: &[&str] = &["elm-stuff"];

impl BuildTool for Elm {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        remove_dirs(&self.path, EPHEMERAL_DIRS, dry_run)
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        status_from_dirs(&self.path, EPHEMERAL_DIRS)
    }
}

impl std::fmt::Display for Elm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Elm")
    }
}
