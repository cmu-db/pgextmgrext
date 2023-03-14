mod cmd_install;
mod cmd_list;
mod download;
mod plugin;
mod resolve_pgxs;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Install(CmdInstall),
    List(CmdList),
}

/// Install extension
#[derive(Parser, Debug)]
pub struct CmdInstall {
    /// The name of the extension (in `plugindb.toml`)
    name: String,
}

/// List all extension in plugindb
#[derive(Parser, Debug)]
pub struct CmdList {}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Install(cmd) => {
            cmd_install::cmd_install(cmd)?;
        }
        Commands::List(_) => {
            cmd_list::cmd_list()?;
        }
    }
    Ok(())
}
