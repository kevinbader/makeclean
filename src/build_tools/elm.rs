use super::{BuildStatus, BuildTool, BuildToolProbe};
use crate::{build_tool_manager::BuildToolManager, fs::dir_size};
use std::{
    fs,
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

static BUILD_AND_DEPS_DIR: &str = "elm-stuff";

impl Elm {
    fn dir(&self, name: &str) -> Option<PathBuf> {
        let dir = self.path.join(name);
        if dir.is_dir() {
            Some(dir)
        } else {
            None
        }
    }
}

impl BuildTool for Elm {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        if let Some(dir) = self.dir(BUILD_AND_DEPS_DIR) {
            if dry_run {
                println!("{}: rm -r {}", self.path.display(), dir.display());
            } else {
                fs::remove_dir_all(dir)?;
            }
        }

        Ok(())
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        let size: u64 = [BUILD_AND_DEPS_DIR]
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

impl std::fmt::Display for Elm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Elm")
    }
}
