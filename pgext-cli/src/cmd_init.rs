use std::path::PathBuf;

use anyhow::Result;

use crate::{config::WorkspaceConfig, CmdInit};

pub fn cmd_init(cmd: CmdInit) -> Result<()> {
    println!("pg_config: {}", cmd.pg_config);
    println!("pg_data: {}", cmd.pg_data);

    if !PathBuf::from(cmd.pg_data.clone())
        .join("postgresql.conf")
        .exists()
    {
        anyhow::bail!("postgresql.conf does not exist in data directory");
    }
    if !PathBuf::from(cmd.pg_config.clone()).exists() {
        anyhow::bail!("pg_config does not exist");
    }
    let config = WorkspaceConfig {
        pg_config: cmd.pg_config,
        pg_data: cmd.pg_data,
    };
    std::fs::write(
        PathBuf::from("pgextworkdir").join("config.toml"),
        toml::to_string_pretty(&config)?,
    )?;
    Ok(())
}
