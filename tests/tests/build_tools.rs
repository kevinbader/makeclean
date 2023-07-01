//! Tests that the build tool probes work and are enabled by default.

use std::process::Command;

use anyhow::{Context, Result};
use assert_cmd::prelude::CommandCargoExt;
use assert_fs::{
    fixture::{ChildPath, PathChild},
    TempDir,
};
use makeclean::project::dto::ProjectDto;

use crate::util::{
    cargo::cargo_init, elm::elm_init, flutter::flutter_init, fs::canonicalized_str,
    gradle::gradle_init, mix::mix_init, npm::npm_init,
};

#[test]
fn recognizes_projects() -> Result<()> {
    type InitFunc = fn(&ChildPath) -> Result<()>;
    let tools: &[(&str, InitFunc)] = &[
        ("Cargo", cargo_init),
        ("Elm", elm_init),
        ("Flutter", flutter_init),
        ("Gradle", gradle_init),
        // ("Maven", maven_init),
        ("Mix", mix_init),
        ("NPM", npm_init),
    ];
    for (build_tool_name, init) in tools {
        let root = TempDir::new()?;

        // E.g. NPM uses the name of the parent folder..
        let project_dir = root.child(format!("{build_tool_name}_project"));

        init(&project_dir)
            .with_context(|| format!("Failed to init {build_tool_name}"))
            .unwrap();

        let output = Command::cargo_bin("makeclean")?
            .args(["--list", "--json"])
            .current_dir(&root)
            .output()?;
        dbg!(String::from_utf8(output.stderr)?);
        assert!(output.status.success());
        let output = String::from_utf8(output.stdout)?;

        // We expect a single line/project
        let project: ProjectDto = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(project.path, canonicalized_str(&project_dir));
        assert_eq!(project.build_tools.len(), 1);
        assert_eq!(project.build_tools[0], *build_tool_name);

        root.close().unwrap();
    }

    Ok(())
}

#[test]
fn ignores_npm_projects_within_node_modules() -> Result<()> {
    let root = TempDir::new()?;
    let project_dir = root.child("npm_project");
    npm_init(&project_dir).with_context(|| "failed to init parent project")?;
    let dep_dir = root.child("node_modules/the_dependency");
    npm_init(&dep_dir).with_context(|| "failed to init dependency project within node_modules")?;

    let output = Command::cargo_bin("makeclean")?
        .args(["--list", "--json"])
        .current_dir(&root)
        .output()?;
    dbg!(String::from_utf8(output.stderr)?);
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout)?;

    // We expect a single line/project
    let project: ProjectDto = serde_json::from_str(output.trim())
        .with_context(|| format!("failed to deserialize '{}'", output.trim()))?;
    assert_eq!(project.path, canonicalized_str(&project_dir));
    assert_eq!(project.build_tools.len(), 1);

    root.close()?;
    Ok(())
}
