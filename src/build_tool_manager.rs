//! Manages build-tool probes that are used to recognize projects.

use std::path::Path;

use tracing::debug;

use crate::build_tools::{
    cargo, elm, flutter, gradle, mix, npm, BuildTool, BuildToolKind, BuildToolProbe,
};

/// Used to identify build tools (projects).
///
/// [`BuildToolManager::probe`] delegates to "probes" that implement
/// [`BuildToolProbe`]. Given a directory, they return an instance of
/// [`BuildTool`] if they recognize a corresponding project at that location.
///
/// Start with an instance that recognizes all known build tools and optionally
/// filter them down:
///
/// ```
/// use makeclean::build_tool_manager::BuildToolManager;
/// use makeclean::build_tools::BuildToolKind::*;
///
/// let mut build_tool_manager = BuildToolManager::default();
/// build_tool_manager.filter(&[Rust, Npm]);
/// ```
///
/// Alternatively, start with an instance with no build tool probe configured
/// and then register additional probes:
///
/// ```
/// use makeclean::build_tool_manager::BuildToolManager;
///
/// let mut build_tool_manager = BuildToolManager::new();
/// makeclean::build_tools::cargo::register(&mut build_tool_manager);
/// makeclean::build_tools::npm::register(&mut build_tool_manager);
/// ```
pub struct BuildToolManager {
    probes: Vec<Box<dyn BuildToolProbe>>,
}

impl Default for BuildToolManager {
    fn default() -> Self {
        let mut build_tool_manager = Self::new();

        cargo::register(&mut build_tool_manager);
        elm::register(&mut build_tool_manager);
        flutter::register(&mut build_tool_manager);
        gradle::register(&mut build_tool_manager);
        mix::register(&mut build_tool_manager);
        npm::register(&mut build_tool_manager);

        // TODO: Activate those as soon as the tests are there:
        // maven::register(&mut build_tool_manager);

        build_tool_manager
    }
}

impl BuildToolManager {
    /// Create a new instance with no probes attached.
    ///
    /// To get an instance with default probes, use `BuildToolManager::default`.
    pub fn new() -> Self {
        Self {
            probes: Default::default(),
        }
    }

    /// Add a build-tool probe.
    ///
    /// Not idempotent - multiple calls will cause the probe to be invoked more
    /// than once.
    pub fn register(&mut self, probe: Box<dyn BuildToolProbe>) {
        self.probes.push(probe);
    }

    /// Filters the current list of probes.
    ///
    /// Any probes that are not related to any of the names given in
    /// `project_types` are discarded.
    pub fn filter(&mut self, project_types: &[BuildToolKind]) {
        let mut probes = Vec::new();
        std::mem::swap(&mut self.probes, &mut probes);

        self.probes = probes
            .into_iter()
            // We keep the probe if it applies to any of the requested types
            .filter(|probe| {
                project_types
                    .iter()
                    .any(|project_type| probe.applies_to(*project_type))
            })
            .collect();

        debug!("build tools filtered: {:?}", &self.probes);
    }

    /// Returns all build tools configured in a given directory.
    pub fn probe(&self, dir: &Path) -> Vec<Box<dyn BuildTool>> {
        self.probes
            .iter()
            .filter_map(|probe| probe.probe(dir))
            .collect()
    }
}
