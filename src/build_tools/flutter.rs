use displaydoc::Display;
use serde::Deserialize;

use super::{remove_dirs, status_from_dirs, BuildStatus, BuildTool, BuildToolKind, BuildToolProbe};
use crate::build_tool_manager::BuildToolManager;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(FlutterProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct FlutterProbe;

impl BuildToolProbe for FlutterProbe {
    fn probe(&self, dir: &Path) -> Option<Box<dyn BuildTool>> {
        let yaml_path = dir.join("pubspec.yaml");
        Pubspec::try_from(yaml_path.as_path()).ok().map(|pubspec| {
            Box::new(Flutter {
                dir: dir.to_owned(),
                pubspec,
            }) as Box<dyn BuildTool>
        })
    }

    fn applies_to(&self, kind: BuildToolKind) -> bool {
        kind == BuildToolKind::Flutter
    }
}

#[derive(Debug, Display)]
/// Flutter
pub struct Flutter {
    dir: PathBuf,
    pubspec: Pubspec,
}

static EPHEMERAL_DIRS: &[&str] = &["build", ".dart_tool"];

impl BuildTool for Flutter {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        // `flutter clean` exists, but according to its documentation it would
        // "Delete the build/ and .dart_tool/ directories" anyway. By doing this
        // directly, we don't require flutter to be installed.

        remove_dirs(&self.dir, EPHEMERAL_DIRS, dry_run)
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        status_from_dirs(&self.dir, EPHEMERAL_DIRS)
    }

    fn project_name(&self) -> Option<anyhow::Result<String>> {
        Some(Ok(self.pubspec.name.clone()))
    }
}

#[derive(Debug, Deserialize)]
struct Pubspec {
    name: String,

    // Increases confidence this is a Flutter project file
    #[serde(rename(deserialize = "version"))]
    _version: String,

    // Increases confidence this is a Flutter project file
    #[serde(rename(deserialize = "flutter"))]
    _flutter: serde_yaml::Value,
}

impl TryFrom<&Path> for Pubspec {
    type Error = anyhow::Error;

    fn try_from(yaml_path: &Path) -> Result<Self, Self::Error> {
        let pubspec: Pubspec = serde_yaml::from_str(&fs::read_to_string(yaml_path)?)?;
        Ok(pubspec)
    }
}
