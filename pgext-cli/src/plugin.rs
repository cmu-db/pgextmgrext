use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// Git download metadata
#[derive(Deserialize, Clone)]
pub struct GitDownload {
  pub url: String,
  pub rev: Option<String>,
  pub sub_path: Option<String>,
}

#[derive(Deserialize, Clone)]
pub enum InstallStrategy {
  /// Default install method, simply call create extension.
  #[serde(rename = "install")]
  Install,
  /// Preload shared library
  #[serde(rename = "preload")]
  Preload,
  /// First preload, then install
  #[serde(rename = "preload+install")]
  PreloadInstall,
  /// load in session shared libraries, then install
  #[serde(rename = "load+install")]
  LoadInstall,
  /// load in session shared libraries
  #[serde(rename = "load")]
  Load,
}

fn default_install_strategy() -> InstallStrategy {
  InstallStrategy::Install
}

#[derive(Deserialize, Clone)]
pub enum CheckStrategy {
  /// Default install method, simply call create extension.
  #[serde(rename = "install")]
  Install,
  /// Don't create extension because the unit test is creating it.
  #[serde(rename = "no-install")]
  NoInstall,
}

fn default_check_strategy() -> CheckStrategy {
  CheckStrategy::Install
}

#[derive(Deserialize, Clone)]
pub struct Plugin {
  pub name: String,
  pub version: String,
  pub download_url_tar: Option<String>,
  pub download_url_zip: Option<String>,
  pub download_git: Option<GitDownload>,
  pub copy_from_contrib: Option<String>,
  pub no_download: Option<bool>,
  pub resolver: String,
  #[serde(default)]
  pub resolver_args: Vec<String>,
  #[serde(default = "default_install_strategy")]
  pub install_strategy: InstallStrategy,
  #[serde(default = "default_check_strategy")]
  pub check_strategy: CheckStrategy,
  #[serde(default)]
  pub dependencies: Vec<String>,
}

#[derive(Deserialize)]
pub struct PluginDb {
  pub plugins: Vec<Plugin>,
}

/// Loads `plugindb` from file
pub fn load_plugin_db() -> Result<PluginDb> {
  let plugindb = std::fs::read_to_string("plugindb.toml").context("Failed to open plugindb.toml")?;
  let plugindb: PluginDb = toml::from_str(&plugindb).context("Failed to parse plugindb.toml")?;
  Ok(plugindb)
}

/// Gets a plugin in `plugindb`
pub fn find_plugin(db: &PluginDb, name: &str) -> Result<Plugin> {
  if let Some(plugin) = db.plugins.iter().find(|x| x.name == name) {
    Ok(plugin.clone())
  } else {
    bail!("Plugin {} not found", name);
  }
}

/// Collects all unique shared preload libraries in the dependency chain
pub fn collect_shared_preload_libraries(db: &PluginDb, plugins: &[Plugin]) -> Vec<String> {
  fn collect_helper(db: &PluginDb, plugin: &Plugin, preloads: &mut HashSet<String>) {
    for extname in plugin.dependencies.iter() {
      let dep = db.plugins.iter().find(|x| &x.name == extname).unwrap().clone();
      collect_helper(db, &dep, preloads);
      if let InstallStrategy::Preload | InstallStrategy::PreloadInstall = dep.install_strategy {
        preloads.insert(dep.name);
      }
    }
  }
  // TODO(yuchen): this need to be ordered + unique (is there a linked hashmap)?
  let mut preloads = HashSet::<String>::new();
  for plugin in plugins.iter() {
    collect_helper(db, plugin, &mut preloads);
    if let InstallStrategy::Preload | InstallStrategy::PreloadInstall = plugin.install_strategy {
      preloads.insert(plugin.name.clone());
    }
  }

  preloads.into_iter().collect::<Vec<String>>()
}
