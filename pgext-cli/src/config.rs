use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::plugin::{collect_shared_preload_libraries, Plugin, PluginDb};

#[derive(Serialize, Deserialize)]
pub struct WorkspaceConfig {
  pub pg_config: String,
  pub pg_data: String,
  pub pg_contrib: String,
}

/// Load the workspace config
pub fn load_workspace_config() -> Result<WorkspaceConfig> {
  let config = std::fs::read_to_string(PathBuf::from("pgextworkdir").join("config.toml"))
    .context("cannot find workspace config, did you run init?")?;
  Ok(toml::from_str(&config)?)
}

/// Edit `postgresql.conf` preload libraries list
pub fn edit_pgconf(db: &PluginDb, config: &WorkspaceConfig, plugins: &[Plugin]) -> Result<Vec<String>> {
  let conf = PathBuf::from(&config.pg_data).join("postgresql.conf");
  let pgconf = std::fs::read_to_string(&conf)?;
  let mut new_pgconf = String::new();
  let shared_preloads = collect_shared_preload_libraries(db, plugins);
  for line in pgconf.lines() {
    if line.starts_with("shared_preload_libraries = ") || line.starts_with("#shared_preload_libraries = ") {
      if shared_preloads.is_empty() {
        writeln!(new_pgconf, "shared_preload_libraries = ''  # modified by pgext")?;
      } else {
        writeln!(
          new_pgconf,
          "shared_preload_libraries = '{}' # modified by pgext",
          shared_preloads.join(",")
        )?;
      }
    } else if line.starts_with("session_preload_libraries = ") || line.starts_with("#session_preload_libraries = ") {
      writeln!(new_pgconf, "session_preload_libraries = ''  # modified by pgext")?;
    } else if line.starts_with("local_preload_libraries = ") || line.starts_with("#local_preload_libraries = ") {
      writeln!(new_pgconf, "local_preload_libraries = ''  # modified by pgext")?;
    } else {
      writeln!(new_pgconf, "{}", line)?;
    }
  }
  std::fs::write(conf, new_pgconf)?;

  Ok(shared_preloads)
}
