#![forbid(unsafe_code)]

use anyhow::Result;

use clap::Parser;

use typescript_tools::{link, lint, make_depend, opts, pin, query};

fn main() -> Result<()> {
    let opts = opts::Opts::parse();

    match opts.subcommand {
        opts::ClapSubCommand::Link(args) => link::link_typescript_project_references(args),
        opts::ClapSubCommand::Pin(args) => pin::pin_version_numbers_in_internal_packages(args),
        opts::ClapSubCommand::MakeDepend(args) => make_depend::make_dependency_makefile(args),
        opts::ClapSubCommand::Query(args) => query::handle_subcommand(args),
        opts::ClapSubCommand::Lint(args) => lint::handle_subcommand(args),
    }
}
