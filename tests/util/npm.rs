use std::{fs, path::Path};

use anyhow::Result;
use assert_fs::fixture::{FileTouch, PathChild};

pub fn npm_init<T>(parent: &T) -> Result<()>
where
    T: PathChild + AsRef<Path>,
{
    fs::create_dir_all(parent.as_ref())?;
    parent.child("package.json").touch()?;
    // parent
    //     .child("node_modules")
    //     .child(".package-lock.json")
    //     .touch()?;
    Ok(())
}
