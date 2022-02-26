use serde::{Deserialize, Serialize};

use crate::{find::vcs::VersionControlSystem, Project};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDto {
    /// The name of project.
    ///
    /// Typically what the first build tool thinks it's called or the name of
    /// the enclosing folder.
    pub name: String,
    /// Where this project is located.
    pub path: String,
    /// The build tools used.
    pub build_tools: Vec<String>,
    /// The VCS, if under version control.
    pub vcs: Option<VcsDto>,
    /// When this project was last modified (most recent commit timestamp).
    pub mtime: String,
}

impl From<&Project> for ProjectDto {
    fn from(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            path: project.path.as_str().to_owned(),
            build_tools: project
                .build_tools
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>(),
            vcs: project.vcs.as_ref().map(VcsDto::from),
            mtime: project.mtime.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VcsDto {
    pub name: String,
    pub root: String,
}

impl From<&VersionControlSystem> for VcsDto {
    fn from(vcs: &VersionControlSystem) -> Self {
        let name = vcs.name().to_owned();
        let root = vcs.root().as_str().to_owned();
        Self { name, root }
    }
}
