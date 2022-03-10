//! Utility functions for file system handling.

use ignore::{gitignore::Gitignore, Match};
use std::path::Path;
use tracing::warn;
use walkdir::WalkDir;

pub(crate) fn dir_size(path: &Path) -> u64 {
    if !path.is_dir() {
        return 0;
    }

    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .fold(0, |acc, m| acc + m.len())
}

#[allow(non_upper_case_globals)]
pub fn format_size(bytes: u64) -> String {
    const KiB: u64 = 1024;
    const MiB: u64 = KiB * 1024;
    const GiB: u64 = MiB * 1024;
    const TiB: u64 = GiB * 1024;
    match bytes {
        n if n < 3 * KiB => format!("{} B", n),
        n if n < 3 * MiB => format!("{} KiB", n / KiB),
        n if n < 3 * GiB => format!("{} MiB", n / MiB),
        n if n < 3 * TiB => format!("{} GiB", n / GiB),
        n => format!("{} TiB", n / TiB),
    }
}

pub(crate) fn is_gitignored(project_dir: &Path, target_dir: &Path) -> bool {
    assert!(project_dir.is_dir());
    assert!(target_dir.is_dir());

    let matchers = [
        (Gitignore::new(project_dir.join(".gitignore")), "local"),
        (Gitignore::global(), "global"),
    ];

    for ((matcher, error), scope) in matchers {
        if let Some(error) = error {
            warn!("Could not create {scope} gitignore matcher: {error}");
            return false;
        }

        match matcher.matched(target_dir, true) {
            Match::Ignore(_) => return true,
            Match::Whitelist(_) => return false,
            Match::None => continue,
        };
    }

    // No match means not whitelisted but also not ignored
    false
}
