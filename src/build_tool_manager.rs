//! Manages build-tool probes that are used to recognize projects.

use std::path::Path;

use tracing::debug;

use crate::build_tools::{
    cargo, elm, flutter, gradle, mix, npm, BuildTool, BuildToolKind, BuildToolProbe,
};

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
