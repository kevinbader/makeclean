use super::{remove_dirs, status_from_dirs, BuildStatus, BuildTool, BuildToolKind, BuildToolProbe};
use crate::build_tool_manager::BuildToolManager;

use displaydoc::Display;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(CargoProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct CargoProbe;

impl BuildToolProbe for CargoProbe {
    fn probe(&self, dir: &Path) -> Option<Box<dyn BuildTool>> {
        let toml_path = dir.join("Cargo.toml");
        CargoToml::try_from(toml_path.as_path()).ok().map(|toml| {
            Box::new(Cargo {
                dir: dir.to_owned(),
                toml,
            }) as Box<dyn BuildTool>
        })
    }

    fn applies_to(&self, kind: BuildToolKind) -> bool {
        use BuildToolKind::*;
        matches!(kind, Cargo | Rust | Rs)
    }
}

#[derive(Debug, Display)]
/// Cargo
pub struct Cargo {
    dir: PathBuf,
    toml: CargoToml,
}

static EPHEMERAL_DIRS: &[&str] = &["target"];

impl BuildTool for Cargo {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        // `cargo clean` exists, but according to its man page:
        // "With no options, cargo clean will delete the entire target directory.".
        // So removing the target directory directly instead of shelling out has
        // the same effect, and also works in case Cargo is not installed on the
        // system.

        remove_dirs(&self.dir, EPHEMERAL_DIRS, dry_run)
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        status_from_dirs(&self.dir, EPHEMERAL_DIRS)
    }

    fn project_name(&self) -> Option<anyhow::Result<String>> {
        Some(Ok(self.toml.package.name.clone()))
    }
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
}

impl TryFrom<&Path> for CargoToml {
    type Error = anyhow::Error;

    fn try_from(toml_path: &Path) -> Result<Self, Self::Error> {
        let cargo_toml: CargoToml = toml::from_str(&fs::read_to_string(toml_path)?)?;
        Ok(cargo_toml)
    }
}
