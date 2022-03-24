use crate::build_tool_manager::BuildToolManager;

use super::{remove_dirs, status_from_dirs, BuildStatus, BuildTool, BuildToolKind, BuildToolProbe};
use displaydoc::Display;
use std::path::{Path, PathBuf};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(GradleProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct GradleProbe;

impl BuildToolProbe for GradleProbe {
    fn probe(&self, dir: &Path) -> Option<Box<dyn BuildTool>> {
        // We expect two files to be present
        let build_gradle = dir.join("build.gradle");
        let settings_gradle = dir.join("settings.gradle");

        if build_gradle.is_file() && settings_gradle.is_file() {
            Some(Box::new(Gradle {
                dir: dir.to_owned(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, kind: BuildToolKind) -> bool {
        kind == BuildToolKind::Gradle
    }
}

#[derive(Debug, Display)]
/// Gradle
pub struct Gradle {
    dir: PathBuf,
}

static EPHEMERAL_DIRS: &[&str] = &["build"];

impl BuildTool for Gradle {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        // `gradle clean`, i.e., the "clean" task, comes with gradle's base plugin. It
        // removes the build directory defined by $buildDir, which defaults to
        // $projectDir/build. Without executing Gradle, it's hard to figure out what
        // $buildDir is set to, but I have yet to see a project that changed this and
        // used the "build" directory for other purposes. Of course, in this case we'd
        // delete the wrong directory, which could cause data loss. I hope that (1)
        // nobody does this anyway, but if they do, (2) they leverage dry-run and (3) use
        // Git to be able to restore the build directory in case this really happened.

        remove_dirs(&self.dir, EPHEMERAL_DIRS, dry_run)
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        status_from_dirs(&self.dir, EPHEMERAL_DIRS)
    }
}
