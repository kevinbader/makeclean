use std::{path::Path, time::SystemTime};
use walkdir::WalkDir;

pub(crate) fn dir_mtime(path: &Path) -> Option<SystemTime> {
    // TODO: If this is a Git repo, use most recent commit timestamp instead? (impl below)

    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .filter_map(|metadata| metadata.modified().ok())
        .max()
}

// fn most_recent_commit_timestamp(
//     repo: &Repository,
// ) -> Result<DateTime<FixedOffset>, ParseProjectError> {
//     // Find the most recently authored branch:
//     let branches = repo.branches(Some(BranchType::Local)).map_err(|e| {
//         ParseProjectError::GitError(
//             repo.workdir().unwrap_or_else(|| repo.path()).to_owned(),
//             e.into(),
//         )
//     })?;
//     let mut commits: Vec<(Time, String, String)> = branches
//         .map(|branch| branch.expect("branch readable").0)
//         .map(|branch| {
//             let branch_name = branch
//                 .name()
//                 .expect("name readable")
//                 .expect("valid utf8")
//                 .to_owned();
//             let commit = branch
//                 .into_reference()
//                 .peel_to_commit()
//                 .expect("branch points at commit");
//             let commit_sha = commit.id().to_string();
//             let commit_time = commit.time();
//             trace!(
//                 "branch: {}{}+{} {} {}",
//                 commit_time.sign(),
//                 commit_time.seconds(),
//                 commit_time.offset_minutes() * 60,
//                 commit_sha,
//                 branch_name
//             );
//             (commit_time, commit_sha, branch_name)
//         })
//         .collect();
//     commits.sort_unstable_by_key(|&(time, ..)| time);
//     let (git_datetime, commit_sha, branch_name) = commits.first().ok_or_else(|| {
//         ParseProjectError::NoCommits(repo.workdir().unwrap_or_else(|| repo.path()).to_owned())
//     })?;
//     let chrono_datetime =
//         FixedOffset::east(git_datetime.offset_minutes() * 60).timestamp(git_datetime.seconds(), 0);

//     debug!(
//         "most recent commit {} on branch {}: {}",
//         &commit_sha[..7],
//         branch_name,
//         chrono_datetime
//     );
//     Ok(chrono_datetime)
// }
