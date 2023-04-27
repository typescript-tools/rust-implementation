#![forbid(unsafe_code)]

use std::io::{self, Write};

use clap::Parser;

use typescript_tools::{link, lint, make_depend, opts, pin, query};

fn main() -> Result<(), anyhow::Error> {
    let args = opts::Opts::parse();

    Ok(match args.subcommand {
        opts::ClapSubCommand::Link(args) => {
            link::link_typescript_project_references(args.root, args.action)?
        }
        opts::ClapSubCommand::Pin(args) => {
            pin::pin_version_numbers_in_internal_packages(args.root, args.action)?
        }
        opts::ClapSubCommand::MakeDepend(args) => make_depend::make_dependency_makefile(args)?,
        opts::ClapSubCommand::Query(args) => match args.subcommand {
            opts::ClapQuerySubCommand::InternalDependencies(args) => {
                let output = query::query_internal_dependencies(args.root, args.format)?;
                writeln!(io::stdout(), "{}", serde_json::to_string_pretty(&output)?)?;
            }
        },
        opts::ClapSubCommand::Lint(args) => match args.subcommand {
            opts::ClapLintSubCommand::DependencyVersion(args) => {
                lint::lint_dependency_version(args.root, &args.dependencies)?
            }
        },
    })
}
