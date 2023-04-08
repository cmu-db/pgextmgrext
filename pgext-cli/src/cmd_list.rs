use anyhow::Result;
use console::style;

use crate::plugin::load_plugin_db;

/// List all extensions in `plugindb`
pub fn cmd_list() -> Result<()> {
  let db = load_plugin_db()?;
  for plugin in db.plugins {
    println!("{}@{}", style(plugin.name).bold(), plugin.version);
  }
  Ok(())
}
