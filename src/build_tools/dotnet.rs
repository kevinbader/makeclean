use super::{BuildStatus, BuildTool, BuildToolProbe};
use crate::{build_tool_manager::BuildToolManager, fs::dir_size};
use std::{
    path::{Path, PathBuf},
};

pub fn register(manager: &mut BuildToolManager) {
    let probe = Box::new(DotnetProbe {});
    manager.register(probe);
}

#[derive(Debug)]
pub struct DotnetProbe;

impl BuildToolProbe for DotnetProbe {
    fn probe(&self, path: &Path) -> Option<Box<dyn BuildTool>> {
        let is_csproj = match path.extension() {
            None => false,
            Some(os_str) => {
                match os_str.to_str() {
                    Some("csproj") => true,
                    _ => false,
                }
            }
        };
        
        if is_csproj {
            Some(Box::new(Dotnet {
                path: path.parent().unwrap().to_path_buf(),
            }))
        } else {
            None
        }
    }

    fn applies_to(&self, name: &str) -> bool {
        let name = name.to_lowercase();
        ["dotnet", "cs", "csharp"].contains(&name.as_str())
    }
}

#[derive(Debug)]
pub struct Dotnet {
    path: PathBuf,
}

impl BuildTool for Dotnet {
    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()> {
        if dry_run {
            println!("{}: rm -r bin", self.path.display());
            println!("{}: rm -r obj", self.path.display());
        } else {
            //fs::remove_dir_all(node_modules)?;
        }
        Ok(())
    }

    fn status(&self) -> anyhow::Result<BuildStatus> {
        let build_dir = self.path.join("bin");
        let obj_dir = self.path.join("obj");
        let status = if build_dir.exists() && obj_dir.exists() {
            let freeable_bytes = dir_size(build_dir.as_ref()) + dir_size(obj_dir.as_ref());
            BuildStatus::Built { freeable_bytes }
        } else {
            BuildStatus::Clean
        };
        Ok(status)
    }
}

impl std::fmt::Display for Dotnet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DOTNET")
    }
}
