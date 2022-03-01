use super::{BuildStatus, BuildTool, BuildToolProbe};
use crate::{build_tool_manager::BuildToolManager, fs::dir_size};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(ElmProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct ElmProbe;

impl BuildToolProbe for ElmProbe {
    fn probe(&self, path: &Utf8Path) -> Option<Box<dyn BuildTool>> {
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
    path: Utf8PathBuf,
}

impl BuildTool for Elm {
    fn status(&self) -> anyhow::Result<BuildStatus> {
        let build_and_deps_dir = self.path.join("elm-stuff");
        let status = if build_and_deps_dir.exists() {
            let freeable_bytes = dir_size(build_and_deps_dir.as_ref());
            BuildStatus::Built { freeable_bytes }
        } else {
            BuildStatus::Clean
        };
        Ok(status)
    }

    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        let build_and_deps_dir = self.path.join("elm-stuff");
        if build_and_deps_dir.exists() {
            assert!(build_and_deps_dir.is_dir());
            if dry_run {
                println!("{}: rm -r {build_and_deps_dir}", self.path);
            } else {
                fs::remove_dir_all(build_and_deps_dir)?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Elm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Elm")
    }
}
