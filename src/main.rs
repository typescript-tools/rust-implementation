#![forbid(unsafe_code)]

use std::io::{self, Write};

use clap::Parser;

use typescript_tools::{
    link, lint, make_depend,
    opts::{self, Action},
    pin, query,
};

// RESUME: why is this not printing with display?
fn main() -> Result<(), anyhow::Error> {
    let args = opts::Opts::parse();

    Ok(match args.subcommand {
        opts::ClapSubCommand::Link(args) => match args.action {
            Action::Modify => link::modify(args.root)?,
            Action::Lint => link::lint(args.root)?,
        },
        opts::ClapSubCommand::Pin(args) => match args.action {
            Action::Modify => pin::modify(args.root)?,
            Action::Lint => pin::lint(args.root)?,
        },
        opts::ClapSubCommand::MakeDepend(args) => make_depend::make_dependency_makefile(args)?,
        opts::ClapSubCommand::Query(args) => match args.subcommand {
            // FEAT: implement internal-dependents
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
