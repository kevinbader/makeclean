use camino::Utf8PathBuf;
use std::path::Path;
use walkdir::WalkDir;

pub(crate) fn canonicalized(path: impl AsRef<Path>) -> anyhow::Result<Utf8PathBuf> {
    let path = path.as_ref().canonicalize()?.try_into()?;
    Ok(path)
}

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
