use super::{remove_dirs, status_from_dirs, BuildStatus, BuildTool, BuildToolKind, BuildToolProbe};
use crate::build_tool_manager::BuildToolManager;
use std::path::{Path, PathBuf};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(ElmProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct ElmProbe;

impl BuildToolProbe for ElmProbe {
    fn probe(&self, dir: &Path) -> Option<Box<dyn BuildTool>> {
        if dir.join("elm.json").is_file() {
            Some(Box::new(Elm {
                dir: dir.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, kind: BuildToolKind) -> bool {
        kind == BuildToolKind::Elm
    }
}

#[derive(Debug)]
pub struct Elm {
    dir: PathBuf,
}

static EPHEMERAL_DIRS: &[&str] = &["elm-stuff"];

impl BuildTool for Elm {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        remove_dirs(&self.dir, EPHEMERAL_DIRS, dry_run)
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        status_from_dirs(&self.dir, EPHEMERAL_DIRS)
    }
}

impl std::fmt::Display for Elm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Elm")
    }
}
