use camino::Utf8Path;
use tracing::debug;

use crate::{
    build_tools::{BuildTool, BuildToolProbe},
    cargo, npm,
};

pub struct BuildToolManager {
    probes: Vec<Box<dyn BuildToolProbe>>,
}

impl Default for BuildToolManager {
    fn default() -> Self {
        let mut build_tool_manager = Self {
            probes: Default::default(),
        };

        cargo::register(&mut build_tool_manager);
        npm::register(&mut build_tool_manager);

        // TODO: Activate those as soon as the tests are there:
        // elm::register(&mut build_tool_manager);
        // gradle::register(&mut build_tool_manager);
        // maven::register(&mut build_tool_manager);
        // mix::register(&mut build_tool_manager);

        build_tool_manager
    }
}

impl BuildToolManager {
    pub fn register(&mut self, probe: Box<dyn BuildToolProbe>) {
        self.probes.push(probe);
    }

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

    pub fn probe(&self, path: &Utf8Path) -> Vec<Box<dyn BuildTool>> {
        self.probes
            .iter()
            .filter_map(|probe| probe.probe(path))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::{elm, gradle, maven, mix};

    use super::*;

    #[test]
    fn filtering_works_with_multiple_project_types() {
        let mut build_tool_manager = BuildToolManager {
            probes: Default::default(),
        };

        cargo::register(&mut build_tool_manager);
        elm::register(&mut build_tool_manager);
        gradle::register(&mut build_tool_manager);
        maven::register(&mut build_tool_manager);
        mix::register(&mut build_tool_manager);
        npm::register(&mut build_tool_manager);

        let project_types = vec!["rs".to_owned(), "elm".to_owned()];
        build_tool_manager.filter(&project_types);

        assert_eq!(build_tool_manager.probes.len(), 2);
        assert!(format!("{:?}", build_tool_manager.probes[0]).starts_with("CargoProbe"));
        assert!(format!("{:?}", build_tool_manager.probes[1]).starts_with("ElmProbe"));
    }
}
