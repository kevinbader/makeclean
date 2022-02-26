pub mod cargo;
pub mod elm;
pub mod gradle;
pub mod maven;
pub mod mix;
pub mod npm;

use camino::Utf8Path;

pub trait BuildToolProbe: std::fmt::Debug {
    fn probe(&self, path: &Utf8Path) -> Option<Box<dyn BuildTool>>;
    fn applies_to(&self, name: &str) -> bool;
}

pub trait BuildTool: std::fmt::Debug + std::fmt::Display {
    fn status(&self) -> anyhow::Result<BuildStatus> {
        Ok(BuildStatus::Unknown)
    }

    fn clean_project(&mut self, dry_run: bool) -> anyhow::Result<()>;

    fn project_name(&self) -> Option<anyhow::Result<String>> {
        None
    }
}

#[derive(Debug)]
pub enum BuildStatus {
    Clean,
    Built { freeable_bytes: u64 },
    Unknown,
}
