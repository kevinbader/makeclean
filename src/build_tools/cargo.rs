use super::{BuildStatus, BuildTool, BuildToolProbe};
use crate::{build_tool_manager::BuildToolManager, fs::dir_size};
use anyhow::{bail, Context};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use std::{fs, process::Command};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(CargoProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub(crate) struct CargoProbe;

impl BuildToolProbe for CargoProbe {
    fn probe(&self, path: &Utf8Path) -> Option<Box<dyn BuildTool>> {
        if path.join("Cargo.toml").is_file() {
            Some(Box::new(Cargo {
                path: path.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, name: &str) -> bool {
        // `name` should already be lowercase, but let's be defensive
        let name = name.to_lowercase();
        ["cargo", "rust", "rs"].contains(&name.as_str())
    }
}

#[derive(Debug)]
pub(crate) struct Cargo {
    path: Utf8PathBuf,
}

impl BuildTool for Cargo {
    fn status(&self) -> anyhow::Result<BuildStatus> {
        let build_dir = self.path.join("target");
        let status = if build_dir.exists() {
            let freeable_bytes = dir_size(build_dir.as_ref());
            BuildStatus::Built { freeable_bytes }
        } else {
            BuildStatus::Clean
        };
        Ok(status)
    }

    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        let mut cmd = Command::new("cargo");
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

    fn project_name(&self) -> Option<anyhow::Result<String>> {
        let toml_path = self.path.join("Cargo.toml");
        Some(read_project_name_from_cargo_toml(&toml_path))
    }
}

impl std::fmt::Display for Cargo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cargo")
    }
}

fn read_project_name_from_cargo_toml(toml_path: &Utf8Path) -> anyhow::Result<String> {
    let cargo_toml: CargoToml = toml::from_str(&fs::read_to_string(toml_path)?)?;
    Ok(cargo_toml.package.name)
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
}
