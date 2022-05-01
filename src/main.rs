#![forbid(unsafe_code)]

use anyhow::Result;

use clap::Parser;

use typescript_tools::link;
use typescript_tools::lint;
use typescript_tools::make_depend;
use typescript_tools::opts;
use typescript_tools::pin;
use typescript_tools::query;

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
