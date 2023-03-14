use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub pg_config: String,
    pub pg_data: String,
    pub pg_contrib: String,
}

pub fn load_workspace_config() -> Result<WorkspaceConfig> {
    let config = std::fs::read_to_string(PathBuf::from("pgextworkdir").join("config.toml"))
        .context("cannot find workspace config, did you run init?")?;
    Ok(toml::from_str(&config)?)
}
