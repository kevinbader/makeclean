use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{bail, Result};
use assert_fs::fixture::{FileTouch, PathChild};
use tracing::warn;

pub fn elm_init<T>(parent: &T) -> Result<()>
where
    T: PathChild + AsRef<Path>,
{
    fs::create_dir_all(parent.as_ref())?;

    let mut proc = match Command::new("elm")
        .arg("init")
        .current_dir(parent.as_ref())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(proc) => proc,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            warn!("failed to exec elm: {}", e);
            // not installed on this system.. let's fake it then
            parent.child("elm.json").touch()?;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    // elm init expects a "y"es (it asks whether to actually init)
    let mut stdin = proc.stdin.take().unwrap();
    stdin.write_all(b"y")?;
    drop(stdin);

    let ecode = proc.wait()?;
    if !ecode.success() {
        bail!("elm init failed with status code {:?}", ecode.code());
    }

    Ok(())
}
