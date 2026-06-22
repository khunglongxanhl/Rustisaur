//! Rustisaur CLI entry point.

mod commands;
mod repl;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use commands::{check, run, version};

#[derive(Parser)]
#[command(name = "Rustisaur")]
#[command(author = "Rustisaur Contributors")]
#[command(version)]
#[command(about = "Rustisaur - A high-performance embedded scripting language")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Rustisaur/Lua script file
    Run {
        /// Path to the script file
        file: String,
    },
    /// Start the interactive REPL
    Repl,
    /// Check script syntax without executing
    Check {
        /// Path to the script file
        file: String,
    },
    /// Show version information
    Version,
    /// Build/bundle a script (placeholder)
    Build {
        /// Path to the script file
        file: String,
        /// Output path
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Run { file } => run::execute(&file),
        Commands::Repl => repl::RexREPL::new()?.run(),
        Commands::Check { file } => check::execute(&file),
        Commands::Version => {
            version::print_version();
            Ok(())
        }
        Commands::Build { file, output } => commands::build::execute(&file, output.as_deref()),
    }
}
