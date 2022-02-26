pub mod git;

use std::{fs, path::Path};

use anyhow::Result;
use assert_fs::fixture::{FileTouch, PathChild};
use chrono::Duration;
use makeclean::ProjectFilter;

pub fn noop_project_filter() -> ProjectFilter {
    ProjectFilter {
        min_age: Duration::days(0),
        status: makeclean::ProjectStatus::Any,
    }
}

// pub fn elm_init<T>(parent: &T) -> Result<()>
// where
//     T: PathChild + AsRef<Path>,
// {
//     fs::create_dir_all(parent.as_ref())?;
//     parent.child("elm.json").touch()?;
//     Ok(())
// }

// pub fn gradle_init<T>(parent: &T) -> Result<()>
// where
//     T: PathChild + AsRef<Path>,
// {
//     fs::create_dir_all(parent.as_ref())?;
//     parent.child("build.gradle").touch()?;
//     Ok(())
// }

// pub fn maven_init<T>(parent: &T) -> Result<()>
// where
//     T: PathChild + AsRef<Path>,
// {
//     fs::create_dir_all(parent.as_ref())?;
//     parent.child("pom.xml").touch()?;
//     Ok(())
// }

// pub fn mix_init<T>(parent: &T) -> Result<()>
// where
//     T: PathChild + AsRef<Path>,
// {
//     fs::create_dir_all(parent.as_ref())?;
//     parent.child("mix.exs").touch()?;
//     Ok(())
// }

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
