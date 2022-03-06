use std::{fs, io, path::Path, process::Command};

use anyhow::{bail, Result};
use assert_fs::fixture::{FileWriteStr, PathChild};
use tracing::warn;

pub fn mix_init<T>(parent: &T) -> Result<()>
where
    T: PathChild + AsRef<Path>,
{
    fs::create_dir_all(parent.as_ref())?;
    match Command::new("mix")
        .args(["new", ".", "--app", "mix_test_project"])
        .current_dir(parent.as_ref())
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => bail!("mix new failed: {:?}", output),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            warn!("failed to exec mix: {}", e);
            // not installed on this system.. let's fake it then
            let mix_exs = r#"
                defmodule MixTestProject.MixProject do
                    use Mix.Project
                
                    def project, do:
                    [
                        app: :mix_test_project,
                        version: "0.1.0",
                        elixir: "~> 1.11",
                        start_permanent: Mix.env() == :prod,
                        deps: []
                    ]
                
                    def application, do: [ extra_applications: [:logger] ]
                end          
                "#;
            parent.child("mix.exs").write_str(mix_exs)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
