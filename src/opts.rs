use std::path::PathBuf;

use clap::{crate_version, ArgEnum, Parser};

#[derive(Parser)]
#[clap(name = "monorepo", version = crate_version!(), author = "Eric Crosson <eric.s.crosson@utexas.edu>")]
pub(crate) struct Opts {
    #[clap(subcommand)]
    pub subcommand: ClapSubCommand,
}

#[derive(Parser)]
pub enum ClapSubCommand {
    #[clap(about = "Pin internal dependencies to locally-declared package versions")]
    Pin(Pin),
    #[clap(about = "Configure TypeScript Project References")]
    Link(Link),
    #[clap(about = "Create GNU Makefile containing package dependency information")]
    MakeDepend(MakeDepend),
    #[clap(about = "Query properties of the current monorepo state")]
    Query(Query),
}

#[derive(Parser)]
pub struct Link {
    /// Path to monorepo root
    #[clap(short, long, default_value = ".")]
    pub root: PathBuf,
    /// Patterns to ignore when resolving lerna package globs
    #[clap(long = "ignore", multiple_values(true))]
    pub ignore: Vec<String>,
    /// Exit with code 1 when project references are not properly configured
    #[clap(long = "check")]
    pub check_only: bool,
}

#[derive(Parser)]
pub struct Pin {
    /// Path to monorepo root
    #[clap(short, long, default_value = ".")]
    pub root: PathBuf,
    /// Patterns to ignore when resolving lerna package globs
    #[clap(long = "ignore", multiple_values(true))]
    pub ignore: Vec<String>,
    /// Exit with code 1 when internal dependencies are not properly pinned
    #[clap(long = "check")]
    pub check_only: bool,
}

#[derive(Parser)]
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
}

#[derive(Parser)]
pub struct Query {
    /// internal-dependencies
    #[clap(subcommand)]
    pub subcommand: ClapQuerySubCommand,
}

#[derive(Parser)]
pub enum ClapQuerySubCommand {
    #[clap(
        about = "Print a JSON object mapping a package name to a list of relative paths to its internal dependencies"
    )]
    InternalDependencies(InternalDependencies),
}

#[derive(Parser)]
pub struct InternalDependencies {
    /// Path to monorepo root
    #[clap(short, long, default_value = ".")]
    pub root: PathBuf,
    /// Patterns to ignore when resolving lerna package globs
    #[clap(long = "ignore", multiple_values(true))]
    pub ignore: Vec<String>,
    /// Format in which to describe internal dependencies (defaults to name)
    #[clap(long = "format", arg_enum, default_value = "name")]
    pub format: InternalDependenciesFormat,
}

#[derive(ArgEnum, Clone)]
pub enum InternalDependenciesFormat {
    Name,
    Path,
}
