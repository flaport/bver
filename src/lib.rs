pub mod bump;
pub mod cast;
pub mod finders;
pub mod git;
pub mod loader;
pub mod schema;
pub mod tui;
pub mod version;

#[cfg(feature = "python")]
mod python;

use std::ffi::OsString;

use bump::bump_version;
use clap::{Parser, Subcommand};
use loader::load_config;

#[derive(Parser)]
#[command(name = "bver")]
#[command(about = "A version management tool")]
#[command(version)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current version
    Current,
    /// Show full config
    Config,
    /// Bump version
    Bump {
        /// Version component (major, minor, patch) or explicit version (e.g. 1.2.3)
        #[arg(default_value = "patch")]
        target: String,

        /// Force git operations (tag, push)
        #[arg(short, long)]
        force: bool,
    },
}

pub fn run() {
    run_from(Cli::parse());
}

pub fn run_from_args<I, T>(args: I)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    run_from(Cli::parse_from(args));
}

fn run_from(cli: Cli) {
    let config = load_config();

    match cli.command {
        Commands::Current => {
            if let Some(config) = config {
                if let Some(version) = config.current_version {
                    println!("{version}");
                } else {
                    eprintln!("No current_version found in config");
                }
            } else {
                eprintln!("No config found");
            }
        }
        Commands::Config => {
            if let Some(config) = config {
                println!("{}", toml::to_string_pretty(&config).unwrap());
            } else {
                eprintln!("No config found");
            }
        }
        Commands::Bump { target, force } => {
            if let Some(config) = config {
                if let Err(e) = bump_version(&config, &target, force) {
                    eprintln!("Error: {e}");
                }
            } else {
                eprintln!("No config found");
            }
        }
    }
}
