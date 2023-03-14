use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use console::style;

use crate::{plugin::load_plugin_db, CmdInstall, CmdInstallAll};

fn create_workdir() -> Result<PathBuf> {
    std::fs::create_dir_all("pgextworkdir")?;
    std::fs::create_dir_all("pgextworkdir/downloads")?;
    std::fs::create_dir_all("pgextworkdir/builds")?;
    Ok(PathBuf::new().join("pgextworkdir"))
}

pub fn cmd_install_all(cmd: CmdInstallAll) -> Result<()> {
    let db = load_plugin_db()?;
    for (idx, plugin) in db.plugins.iter().enumerate() {
        println!("{}/{}", idx + 1, db.plugins.len());
        cmd_install(CmdInstall {
            name: plugin.name.clone(),
            verbose: cmd.verbose,
        })?;
    }
    Ok(())
}

pub fn cmd_install(cmd: CmdInstall) -> Result<()> {
    let db = load_plugin_db()?;
    let workdir = create_workdir()?;
    let pg_config = std::env::var("PG_CONFIG").context("cannot find PG_CONFIG env variable")?;
    if let Some(plugin) = db.plugins.iter().find(|x| x.name == cmd.name) {
        println!(
            "{} {}@{}",
            style("Installing").blue().bold(),
            style(&plugin.name).bold(),
            plugin.version
        );
        let name_tag = format!("{}@{}", plugin.name, plugin.version);

        let build_dir = workdir.join("builds").join(&name_tag);

        if let Some(url) = &plugin.download_url_zip {
            let download_path = workdir.join("downloads").join(format!("{}.zip", name_tag));
            crate::download::download_zip(
                url.to_string(),
                &download_path,
                &build_dir,
                cmd.verbose,
            )?;
        } else if let Some(url) = &plugin.download_url_tar {
            let download_path = workdir
                .join("downloads")
                .join(format!("{}.tar.gz", name_tag));
            crate::download::download_tar(
                url.to_string(),
                &download_path,
                &build_dir,
                cmd.verbose,
            )?;
        } else if let Some(src) = &plugin.download_git {
            let download_path = workdir.join("downloads").join(name_tag);
            crate::download::download_git(src, &download_path, &build_dir, cmd.verbose)?;
        } else if let Some(true) = &plugin.no_download {
            println!("{} {}", style("Skipping Download").bold().blue(), name_tag);
        } else {
            return Err(anyhow!("No download url found in plugindb.toml"));
        }

        match plugin.resolver.as_str() {
            "pgxs" => crate::resolve_pgxs::resolve_build_pgxs(
                plugin,
                &build_dir,
                &pg_config,
                cmd.verbose,
            )?,
            "pgsrctree" => {
                println!(
                    "{}: in Postgres source tree",
                    style("Skipping Build").bold().blue()
                );
            }
            _ => return Err(anyhow!("Unknown resolver: {}", plugin.resolver)),
        }

        println!(
            "{} {}@{}",
            style("Installed").bold().green(),
            style(&plugin.name).bold(),
            plugin.version
        );

        Ok(())
    } else {
        Err(anyhow!(
            "Could not find extension {} in plugindb.toml",
            cmd.name
        ))
    }
}
