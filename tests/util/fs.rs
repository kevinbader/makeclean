use std::path::Path;

use camino::Utf8PathBuf;

pub fn canonicalized(path: impl AsRef<Path>) -> anyhow::Result<Utf8PathBuf> {
    let path = path.as_ref().canonicalize()?.try_into()?;
    Ok(path)
}

pub fn canonicalized_str(path: impl AsRef<Path>) -> String {
    canonicalized(path).unwrap().as_str().to_owned()
}
