use std::path::PathBuf;

use clap::{crate_version, ArgAction, Parser, ValueEnum};
use typescript_tools::query;

#[derive(Debug, Parser)]
#[clap(name = "monorepo", version = crate_version!(), author = "Eric Crosson <eric.s.crosson@utexas.edu>")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcommand: ClapSubCommand,
}

#[derive(Debug, Parser)]
pub enum ClapSubCommand {
    #[clap(about = "Pin internal dependencies to locally-declared package versions")]
    Pin(Pin),

    #[clap(about = "Configure TypeScript Project References")]
    Link(Link),

    #[clap(about = "Create GNU Makefile containing package dependency information")]
    MakeDepend(MakeDepend),

    #[clap(about = "Query properties of the current monorepo state")]
    Query(Query),

    #[clap(about = "Lint internal packages for consistent use of external dependency versions")]
    Lint(Lint),
}

#[derive(Debug, Parser)]
pub struct Link {
    /// Path to monorepo root
    #[clap(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Modify tsconfig.json files as necessary to restore link invariant
    #[clap(long = "write", action = ArgAction::SetTrue)]
    pub action: Action,
}

#[derive(Debug, Parser)]
pub struct Pin {
    /// Path to monorepo root
    #[clap(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Modify package.json files as necessary to restore pin invariant
    #[clap(long = "write", action = ArgAction::SetTrue)]
    pub action: Action,
}

#[derive(Debug, Parser)]
pub struct MakeDepend {
    /// Path to monorepo root
    #[clap(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Directory to package for which to calculate dependencies
    #[clap(long)]
    pub package_directory: PathBuf,

    /// Output file, relative to the package directory
    #[clap(long)]
    pub output_file: PathBuf,

    /// Include GNU make target for creating package archive with npm pack
    #[clap(long)]
    pub create_pack_target: bool,
}

#[derive(Debug, Parser)]
pub struct Query {
    #[clap(subcommand)]
    pub subcommand: ClapQuerySubCommand,
}

#[derive(Debug, Parser)]
pub enum ClapQuerySubCommand {
    #[clap(
        about = "Print a JSON object mapping a package name to a list of relative paths to its internal dependencies"
    )]
    InternalDependencies(InternalDependencies),
}

#[derive(ValueEnum, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum InternalDependenciesFormat {
    Name,
    Path,
}

impl From<InternalDependenciesFormat> for query::InternalDependenciesFormat {
    fn from(value: InternalDependenciesFormat) -> Self {
        match value {
            InternalDependenciesFormat::Name => Self::Name,
            InternalDependenciesFormat::Path => Self::Path,
        }
    }
}

#[derive(Debug, Parser)]
pub struct InternalDependencies {
    /// Path to monorepo root
    #[clap(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Format in which to describe internal dependencies (defaults to name)
    #[clap(long = "format", value_enum, default_value = "name")]
    pub format: InternalDependenciesFormat,
}

#[derive(Debug, Parser)]
pub struct Lint {
    #[clap(subcommand)]
    pub subcommand: ClapLintSubCommand,
}

#[derive(Debug, Parser)]
pub enum ClapLintSubCommand {
    #[clap(about = "Lint the used versions of an external dependency for consistency")]
    DependencyVersion(DependencyVersion),
}

#[derive(Debug, Parser)]
pub struct DependencyVersion {
    /// Path to monorepo root
    #[clap(short, long, default_value = ".")]
    pub root: PathBuf,

    /// External dependency to lint for consistency of version used
    #[clap(short, long = "dependency")]
    pub dependencies: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Action {
    Modify,
    Lint,
}

impl From<&str> for Action {
    fn from(value: &str) -> Self {
        match value {
            "true" => Self::Modify,
            "false" => Self::Lint,
            _ => unreachable!(),
        }
    }
}
