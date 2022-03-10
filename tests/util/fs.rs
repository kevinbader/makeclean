use std::path::Path;

pub fn canonicalized_str(path: impl AsRef<Path>) -> String {
    path.as_ref().canonicalize().unwrap().display().to_string()
}
