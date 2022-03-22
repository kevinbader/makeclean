use super::{remove_dirs, status_from_dirs, BuildStatus, BuildTool, BuildToolKind, BuildToolProbe};
use crate::build_tool_manager::BuildToolManager;
use std::path::{Path, PathBuf};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(MixProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct MixProbe;

impl BuildToolProbe for MixProbe {
    fn probe(&self, dir: &Path) -> Option<Box<dyn BuildTool>> {
        if dir.join("mix.exs").is_file() {
            Some(Box::new(Mix::new(dir)))
        } else {
            None
        }
    }

    fn applies_to(&self, kind: BuildToolKind) -> bool {
        use BuildToolKind::*;
        matches!(kind, Mix | Elixir | Ex | Exs)
    }
}

#[derive(Debug)]
pub struct Mix {
    dir: PathBuf,
}

impl Mix {
    fn new(path: &Path) -> Self {
        Self {
            dir: path.to_owned(),
        }
    }
}

static EPHEMERAL_DIRS: &[&str] = &["_build", "deps", ".elixir_ls"];

impl BuildTool for Mix {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        // `mix clean --deps` exists, but
        // - it needs to be installed
        // - it needs to match the version used in the project
        // - it doesn't remove everything, so projects are still reported un-cleaned
        //
        // So instead, we're simply deleting the well-known directories, which
        // works just as well (better?), is faster, and doesn't require mix to
        // be installed.

        remove_dirs(&self.dir, EPHEMERAL_DIRS, dry_run)
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        status_from_dirs(&self.dir, EPHEMERAL_DIRS)
    }

    fn project_name(&self) -> Option<anyhow::Result<String>> {
        // mix.exs, which contains the project name, is not easy to parse without Elixir.
        // While `mix run -e 'IO.puts(Mix.Project.config[:app])'` would work, it would
        // also compile the application, which is of course an unintended side effect.
        // To prevent false positives, we don't even try.
        None
    }
}

impl std::fmt::Display for Mix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mix")
    }
}

#[cfg(test)]
mod test {
    use assert_fs::{
        fixture::{FileWriteStr, PathChild},
        TempDir,
    };

    use super::*;

    #[test]
    fn elixir_ls_cache_is_removed_even_if_not_gitignored() {
        let root = TempDir::new().unwrap();

        root.child("normal")
            .child(".elixir_ls")
            .child("dummy")
            .write_str("dummy")
            .unwrap();
        root.child("normal")
            .child(".gitignore")
            .write_str(".elixir_ls/")
            .unwrap();

        root.child("not-ignored")
            .child(".elixir_ls")
            .child("dummy")
            .write_str("dummy")
            .unwrap();

        // In the normal case, status should report back that the project is not clean:
        let normal_status = Mix::new(&root.child("normal")).status().unwrap();
        assert!(matches!(normal_status, BuildStatus::Built{freeable_bytes} if freeable_bytes > 0));

        // If not ignored, the behavior is the same:
        let not_ignored_status = Mix::new(&root.child("not-ignored")).status().unwrap();
        assert!(
            matches!(not_ignored_status, BuildStatus::Built{freeable_bytes} if freeable_bytes > 0)
        );
    }
}
