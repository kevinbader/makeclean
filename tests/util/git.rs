use std::{fs, io::Write, path::Path};

use anyhow::Result;
use assert_fs::fixture::PathChild;
use git2::{Commit, Index, IndexAddOption, Repository, Signature};

/// Initializes a Git repository at the given location.
///
/// `gitignore`: How the `.gitignore` file should look like. May contain
/// newlines to set multiple rules.
///
/// `commit`: Whether to do `git add .` and `git commit` afterwards.
pub fn git_init<T>(parent: &T, gitignore: &str, commit: bool)
where
    T: PathChild + AsRef<Path>,
{
    fs::create_dir_all(parent.as_ref()).unwrap();
    let repo = Repository::init(&parent).unwrap();

    if !gitignore.is_empty() {
        let gitignore_path = parent.child(".gitignore");
        let mut gitignore_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(gitignore_path)
            .unwrap();
        gitignore_file.write_all(gitignore.as_bytes()).unwrap();
    }

    if commit {
        let mut index = git_add(&repo, &["."]).unwrap();
        git_commit(&repo, &mut index, "test").unwrap();
    }
}

// see https://libgit2.org/libgit2/ex/HEAD/add.html
fn git_add(repo: &Repository, pathspecs: &[&str]) -> Result<Index> {
    let mut index = repo.index()?;
    index.add_all(pathspecs, IndexAddOption::DEFAULT, None)?;
    index.write()?;

    Ok(index)
}

// see https://libgit2.org/libgit2/ex/HEAD/commit.html
fn git_commit(repo: &Repository, index: &mut Index, message: &str) -> Result<()> {
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    let signature = Signature::now("makeclean-test", "makeclean-test@example.com")?;
    let author = &signature;
    let committer = &signature;

    let parents = match repo.head() {
        Ok(head) => vec![head.peel_to_commit()?],
        Err(e) => {
            if e.code() == git2::ErrorCode::UnbornBranch {
                // No commits yet - HEAD does not exist
                vec![]
            } else {
                return Err(e.into());
            }
        }
    };
    let parents: Vec<&Commit> = parents.iter().collect();

    repo.commit(Some("HEAD"), author, committer, message, &tree, &parents)?;

    Ok(())
}
