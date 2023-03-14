mod cmd_init;
mod cmd_install;
mod cmd_list;
mod cmd_test;
mod config;
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
    Init(CmdInit),
    Install(CmdInstall),
    InstallAll(CmdInstallAll),
    List(CmdList),
    Test(CmdTest),
    TestAll(CmdTestAll),
}

/// Init workspace
#[derive(Parser, Debug)]
pub struct CmdInit {
    /// Path to `pg_config`
    pg_config: String,
    /// Directory that stores Postgres config and data
    pg_data: String,
}

/// Install extension
#[derive(Parser, Debug)]
pub struct CmdInstall {
    /// The name of the extension (in `plugindb.toml`)
    name: String,
    /// Enable verbose mode
    #[clap(short, long)]
    verbose: bool,
}

/// Install all extensions in plugindb
#[derive(Parser, Debug)]
pub struct CmdInstallAll {
    /// Enable verbose mode
    #[clap(short, long)]
    verbose: bool,
}

/// List all extension in plugindb
#[derive(Parser, Debug)]
pub struct CmdList {}

/// Install extension
#[derive(Parser, Debug)]
pub struct CmdTest {
    /// The name of the extension (in `plugindb.toml`)
    name: String,
}

/// Install all extensions in plugindb
#[derive(Parser, Debug)]
pub struct CmdTestAll {}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Init(cmd) => {
            cmd_init::cmd_init(cmd)?;
        }
        Commands::Install(cmd) => {
            cmd_install::cmd_install(cmd)?;
        }
        Commands::InstallAll(cmd) => {
            cmd_install::cmd_install_all(cmd)?;
        }
        Commands::List(_) => {
            cmd_list::cmd_list()?;
        }
        Commands::Test(cmd) => {
            cmd_test::cmd_test(cmd)?;
        }
        Commands::TestAll(cmd) => {
            cmd_test::cmd_test_all(cmd)?;
        }
    }
    Ok(())
}
