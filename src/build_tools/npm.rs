use displaydoc::Display;
use tracing::debug;

use super::{remove_dirs, status_from_dirs, BuildStatus, BuildTool, BuildToolKind, BuildToolProbe};
use crate::build_tool_manager::BuildToolManager;
use std::path::{Path, PathBuf};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(NpmProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct NpmProbe;

impl BuildToolProbe for NpmProbe {
    fn probe(&self, dir: &Path) -> Option<Box<dyn BuildTool>> {
        // If we're within a node_modules directory, we assume this is a
        // dependency and shouldn't be cleansed by itself.
        if dir.components().any(|x| x.as_os_str() == "node_modules") {
            debug!("ignoring directory within node_modules dir at {:?}", dir);
            return None;
        }

        if dir.join("package.json").is_file() {
            Some(Box::new(Npm {
                dir: dir.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, kind: BuildToolKind) -> bool {
        kind == BuildToolKind::Npm
    }
}

#[derive(Debug, Display)]
/// NPM
pub struct Npm {
    dir: PathBuf,
}

static EPHEMERAL_DIRS: &[&str] = &["node_modules"];

impl BuildTool for Npm {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        // TODO: also delete build directory, depending on the language(s) used
        remove_dirs(&self.dir, EPHEMERAL_DIRS, dry_run)
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        status_from_dirs(&self.dir, EPHEMERAL_DIRS)
    }
}
