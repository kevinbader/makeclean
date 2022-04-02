use tracing::trace;

use super::Project;

impl Project {
    /// Invokes `clean` on each of the build tools used in this project.
    pub fn clean(&mut self, dry_run: bool) -> anyhow::Result<()> {
        trace!(?self.path, "cleaning project");
        assert!(!self.build_tools.is_empty());
        for build_tool in self.build_tools.iter_mut() {
            build_tool.clean_project(dry_run)?;
        }
        Ok(())
    }
}
