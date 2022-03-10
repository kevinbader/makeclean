//! Manages build-tool probes that are used to recognize projects.

use std::path::Path;

use tracing::{debug, warn};

use crate::build_tools::{cargo, elm, mix, npm, BuildTool, BuildToolProbe};

pub struct BuildToolManager {
    probes: Vec<Box<dyn BuildToolProbe>>,
}

#[cfg(test)]
#[allow(clippy::derivable_impls)]
impl Default for BuildToolManager {
    fn default() -> Self {
        Self {
            probes: Default::default(),
        }
    }
}

impl BuildToolManager {
    pub fn with_readonly_probes() -> Self {
        let mut build_tool_manager = Self {
            probes: Default::default(),
        };

        if let Err(e) = cargo::register(&mut build_tool_manager, true) {
            warn!("Cargo disabled: {e}");
        }

        elm::register(&mut build_tool_manager);

        if let Err(e) = mix::register(&mut build_tool_manager, true) {
            warn!("Mix disabled: {e}");
        }

        npm::register(&mut build_tool_manager);

        // TODO: Activate those as soon as the tests are there:
        // gradle::register(&mut build_tool_manager);
        // maven::register(&mut build_tool_manager);

        build_tool_manager
    }

    pub fn with_readwrite_probes() -> Self {
        let mut build_tool_manager = Self {
            probes: Default::default(),
        };

        if let Err(e) = cargo::register(&mut build_tool_manager, false) {
            warn!("Cargo disabled: {e}");
        }

        elm::register(&mut build_tool_manager);

        if let Err(e) = mix::register(&mut build_tool_manager, false) {
            warn!("Mix disabled: {e}");
        }

        npm::register(&mut build_tool_manager);

        // TODO: Activate those as soon as the tests are there:
        // gradle::register(&mut build_tool_manager);
        // maven::register(&mut build_tool_manager);

        build_tool_manager
    }
}

impl BuildToolManager {
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
    pub fn filter(&mut self, project_types: &[String]) {
        let mut probes = Vec::new();
        std::mem::swap(&mut self.probes, &mut probes);

        self.probes = probes
            .into_iter()
            // We keep the probe if it applies to any of the requested types
            .filter(|probe| {
                project_types
                    .iter()
                    .any(|project_type| probe.applies_to(project_type))
            })
            .collect();

        debug!("build tools filtered: {:?}", &self.probes);
    }

    /// Returns all build tools that apply to a given location.
    pub fn probe(&self, path: &Path) -> Vec<Box<dyn BuildTool>> {
        self.probes
            .iter()
            .filter_map(|probe| probe.probe(path))
            .collect()
    }
}
