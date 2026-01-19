mod bump;
mod finders;
mod loader;
mod schema;

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
        /// Version component to bump: major, minor, or patch
        #[arg(default_value = "patch")]
        component: String,
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
        Commands::Bump { component } => {
            if let Some(config) = config {
                if let Err(e) = bump_version(&config, &component) {
                    eprintln!("Error: {e}");
                }
            } else {
                eprintln!("No config found");
            }
        }
    }
}
