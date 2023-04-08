use std::path::PathBuf;

use anyhow::Result;

use crate::config::WorkspaceConfig;
use crate::CmdInit;

/// Initialze the workspace
pub fn cmd_init(cmd: CmdInit) -> Result<()> {
  println!("pg_config: {}", cmd.pg_config);
  println!("pg_data: {}", cmd.pg_data);
  println!("pg_contrib: {}", cmd.pg_contrib);

  if !PathBuf::from(cmd.pg_data.clone()).join("postgresql.conf").exists() {
    anyhow::bail!("postgresql.conf does not exist in data directory");
  }
  if !PathBuf::from(cmd.pg_config.clone()).exists() {
    anyhow::bail!("pg_config does not exist");
  }
  let config = WorkspaceConfig {
    pg_config: cmd.pg_config,
    pg_data: cmd.pg_data,
    pg_contrib: cmd.pg_contrib,
  };
  // saving config
  std::fs::write(
    PathBuf::from("pgextworkdir").join("config.toml"),
    toml::to_string_pretty(&config)?,
  )?;
  Ok(())
}
