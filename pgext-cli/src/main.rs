mod cmd_init;
mod cmd_install;
mod cmd_list;
mod cmd_test;
mod config;
mod download;
mod plugin;
mod resolve_pgxs;
mod test_control;
use anyhow::Result;
use clap::{Parser, Subcommand};

/// PgExt - A PostgresSQL extension installer tool
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
  InstallHook(CmdInstallHook),
  InstallAll(CmdInstallAll),
  List(CmdList),
  Test(CmdTest),
  TestSingle(CmdTestSingle),
  TestAll(CmdTestAll),
  Demo(CmdDemo),
}

/// Demo
#[derive(Parser, Debug)]
pub struct CmdDemo {
  /// The name of the extension (in `plugindb.toml`)
  name: String,
}

/// Init workspace
#[derive(Parser, Debug)]
pub struct CmdInit {
  /// Path to `pg_config`
  pg_config: String,
  /// Directory that stores Postgres config and data
  pg_data: String,
  /// Source code contrib directory
  pg_contrib: String,
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

/// Install show_hooks extension
#[derive(Parser, Debug)]
pub struct CmdInstallHook {}

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
pub struct CmdTestSingle {
  /// The name of the extension (in `plugindb.toml`)
  name: String,
  /// Run installchecks
  #[clap(long)]
  check: bool,
}

/// Testing compatibility of a list of extensions
#[derive(Parser, Debug)]
pub struct CmdTest {
  /// extension names in plugindb
  exts: Vec<String>,
  /// Run installchecks
  #[clap(long)]
  check_last: bool,
  /// Run custom SQLs after installing all extensions
  #[clap(long)]
  run_custom_sql: bool,
}

/// Install all extensions in plugindb
#[derive(Parser, Debug)]
pub struct CmdTestAll {
  /// Dump data to file
  #[clap(long)]
  dump_to: Option<String>,
  #[clap(long)]
  check: bool,
}

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
      cmd_test::cmd_test(cmd, None)?;
    }
    Commands::TestSingle(cmd) => {
      cmd_test::cmd_test_single(cmd, None)?;
    }
    Commands::TestAll(cmd) => {
      cmd_test::cmd_test_all(cmd)?;
    }
    Commands::InstallHook(_) => {
      cmd_install::cmd_install_hook()?;
    }
    Commands::Demo(cmd) => cmd_test::cmd_demo(cmd)?,
  }
  Ok(())
}
