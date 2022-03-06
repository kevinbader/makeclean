use ignore::WalkBuilder;
use std::{path::Path, time::SystemTime};

pub(crate) fn dir_mtime(path: &Path) -> Option<SystemTime> {
    WalkBuilder::new(path)
        .standard_filters(true)
        .hidden(false)
        .build()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .filter_map(|metadata| metadata.modified().ok())
        .max()
}
