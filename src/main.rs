mod finders;
mod loader;
mod schema;

use clap::{Parser, Subcommand};
use finders::find_project_root;
use loader::load_config;

#[derive(Parser)]
#[command(name = "vrsn")]
#[command(about = "A version management tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current version
    Current,
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
        Some(Commands::Current) => {
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
        Some(Commands::Bump { component }) => {
            println!("Bump {component} (not yet implemented)");
        }
        None => {
            println!("project_root: {:?}", find_project_root());
            println!("config: {config:?}");
        }
    }
}
