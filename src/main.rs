#![forbid(unsafe_code)]

use std::io::{self, Write};

use clap::Parser;

mod little_anyhow;
mod opts;

use opts::Action;
use typescript_tools::{link, lint, make_depend, pin, query};

// RESUME: why is this not printing with display?
fn main() -> Result<(), little_anyhow::Error> {
    let args = opts::Opts::parse();

    match args.subcommand {
        opts::ClapSubCommand::Link(args) => match args.action {
            Action::Modify => link::modify(args.root)?,
            Action::Lint => link::lint(args.root)?,
        },
        opts::ClapSubCommand::Pin(args) => match args.action {
            Action::Modify => pin::modify(args.root)?,
            Action::Lint => pin::lint(args.root)?,
        },
        opts::ClapSubCommand::MakeDepend(args) => make_depend::make_dependency_makefile(
            &args.root,
            &args.package_directory,
            &args.output_file,
            args.create_pack_target,
        )?,
        opts::ClapSubCommand::Query(args) => match args.subcommand {
            // FEAT: implement internal-dependents
            opts::ClapQuerySubCommand::InternalDependencies(args) => {
                let output = query::query_internal_dependencies(args.root, args.format.into())?;
                writeln!(io::stdout(), "{}", serde_json::to_string_pretty(&output)?)?;
            }
        },
        opts::ClapSubCommand::Lint(args) => match args.subcommand {
            opts::ClapLintSubCommand::DependencyVersion(args) => {
                lint::lint_dependency_version(args.root, &args.dependencies)?
            }
        },
    };
    Ok(())
}
