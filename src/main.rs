mod bump;
mod cast;
mod finders;
mod loader;
mod schema;
mod version;

use bump::bump_version;
use clap::{Parser, Subcommand};
use loader::load_config;

#[derive(Parser)]
#[command(name = "vrsn")]
#[command(about = "A version management tool")]
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
    },
}

fn main() {
    let cli = Cli::parse();

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
        Commands::Bump { target } => {
            if let Some(config) = config {
                if let Err(e) = bump_version(&config, &target) {
                    eprintln!("Error: {e}");
                }
            } else {
                eprintln!("No config found");
            }
        }
    }
}
