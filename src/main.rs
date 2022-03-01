use clap::StructOpt;
use std::{env, panic};

use makeclean::{build_tool_manager::BuildToolManager, Cli};

fn main() -> anyhow::Result<()> {
    setup_panic_hooks();
    setup_logging();

    let cli = Cli::parse();

    let mut build_tool_manager = BuildToolManager::default();
    let project_types = &cli.types;
    if !project_types.is_empty() {
        build_tool_manager.filter(project_types);
    }

    if cli.list {
        makeclean::list(cli, build_tool_manager)
    } else {
        makeclean::clean(cli, build_tool_manager)
    }
}

fn setup_panic_hooks() {
    let meta = human_panic::Metadata {
        version: env!("CARGO_PKG_VERSION").into(),
        name: env!("CARGO_PKG_NAME").into(),
        authors: env!("CARGO_PKG_AUTHORS").replace(':', ", ").into(),
        homepage: env!("CARGO_PKG_HOMEPAGE").into(),
    };

    let _default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
        // // First call the default hook that prints to standard error.
        // default_hook(info);

        // Then call human_panic.
        let file_path = human_panic::handle_dump(&meta, info);
        human_panic::print_msg(file_path, &meta)
            .expect("human-panic: printing error message to console failed");
    }));
}

pub fn setup_logging() {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1")
    }
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    tracing_subscriber::fmt::init();
}
