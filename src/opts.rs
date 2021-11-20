use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(version = "1.0", author = "Eric Crosson <eric.s.crosson@utexas.edu>")]
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
}

#[derive(Parser)]
pub struct Link {
    #[clap(short, long, about = "Path to monorepo root")]
    pub root: PathBuf,
    #[clap(
        long = "ignore",
        multiple_values(true),
        about = "Patterns to ignore when resolving lerna package globs"
    )]
    pub ignore: Vec<String>,
    #[clap(
        long = "check",
        about = "Exit with code 1 when project references are not properly configured"
    )]
    pub check_only: bool,
}

#[derive(Parser)]
pub struct Pin {
    #[clap(short, long, about = "Path to monorepo root")]
    pub root: PathBuf,
    #[clap(
        long = "ignore",
        multiple_values(true),
        about = "Patterns to ignore when resolving lerna package globs"
    )]
    pub ignore: Vec<String>,
    #[clap(
        long = "check",
        about = "Exit with code 1 when internal dependencies are not properly pinned"
    )]
    pub check_only: bool,
}
