use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GitDownload {
    pub url: String,
    pub rev: Option<String>,
    pub sub_path: Option<String>,
}

#[derive(Deserialize)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub download_url_tar: Option<String>,
    pub download_url_zip: Option<String>,
    pub download_git: Option<GitDownload>,
    pub no_download: Option<bool>,
    pub resolver: String,
    #[serde(default)]
    pub resolver_args: Vec<String>,
}

#[derive(Deserialize)]
pub struct PluginDb {
    pub plugins: Vec<Plugin>,
}

pub fn load_plugin_db() -> Result<PluginDb> {
    let plugindb =
        std::fs::read_to_string("plugindb.toml").context("Failed to open plugindb.toml")?;
    let plugindb: PluginDb = toml::from_str(&plugindb).context("Failed to parse plugindb.toml")?;
    Ok(plugindb)
}
