use anyhow::bail;
use git2::Repository;
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum VersionControlSystem {
    Git(Git),
}

impl VersionControlSystem {
    /// Check whether the given path is under version control.
    ///
    /// Returns either the VCS or None, or an `Err` if something unexpected happened.
    ///
    /// In theory there could be multiple VCS managing a single directory, but
    /// arguably this is an edge case. We simply returns the first VCS that matches.
    pub(crate) fn try_from(path: &Path) -> anyhow::Result<Option<Self>> {
        if let Some(git) = Git::try_from(path)? {
            return Ok(Some(Self::Git(git)));
        }

        // TODO: Support for Mercurial and SVN.
        Ok(None)
    }

    pub fn name(&self) -> &'static str {
        use VersionControlSystem::*;
        match *self {
            Git(_) => "Git",
        }
    }

    pub fn root(&self) -> PathBuf {
        use VersionControlSystem::*;
        match *self {
            Git(ref git) => git.root(),
        }
    }
}

pub struct Git {
    repo: Repository,
}

impl Git {
    fn try_from(path: &Path) -> anyhow::Result<Option<Self>> {
        // In deep repositories this might be pretty expensive, as it searches
        // up the directory hierarchy, and the same repository is opened again
        // in every directory of the working copy.
        match Repository::discover(path) {
            Ok(repo) => Ok(Some(Git { repo })),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => bail!("Failed to check for Git repository at {:?}: {}", path, e),
        }
    }

    fn root(&self) -> PathBuf {
        self.repo
            .workdir()
            .unwrap_or_else(|| self.repo.path())
            .canonicalize()
            .unwrap()
    }
}

impl fmt::Debug for Git {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Git").field("root", &self.root()).finish()
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use assert_fs::{fixture::ChildPath, prelude::*, TempDir};

    fn path_of(path: &ChildPath) -> PathBuf {
        path.path().canonicalize().unwrap()
    }

    #[test]
    fn works_in_the_subdirectory_of_a_git_repo() -> anyhow::Result<()> {
        let root = TempDir::new()?;
        let level0_foo = root.child("foo");
        let level1_bar = level0_foo.child("bar");
        let level1_ignored_dir = level0_foo.child("ignored_dir");
        level1_bar.create_dir_all()?;
        level1_ignored_dir.create_dir_all()?;

        root.child(".gitignore").write_str("/foo/ignored_dir")?;
        let _ = Repository::init(root.path())?;

        // We operate in the "foo" subdirectory
        let vcs = VersionControlSystem::try_from(&path_of(&level0_foo))
            .unwrap()
            .unwrap();
        assert_eq!(vcs.name(), "Git");

        Ok(())
    }
}
