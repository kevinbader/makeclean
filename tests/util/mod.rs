pub mod cargo;
pub mod elm;
pub mod flutter;
pub mod fs;
pub mod git;
pub mod mix;
pub mod npm;

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
