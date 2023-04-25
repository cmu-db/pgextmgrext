use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use console::style;
use duct::cmd;
use postgres::{Client, NoTls};

use crate::config::load_workspace_config;
use crate::plugin::load_plugin_db;
use crate::{CmdInstall, CmdInstallAll};

/// Create work directory if not exists
pub fn create_workdir() -> Result<PathBuf> {
  std::fs::create_dir_all("pgextworkdir")?;
  std::fs::create_dir_all("pgextworkdir/downloads")?;
  std::fs::create_dir_all("pgextworkdir/builds")?;
  Ok(PathBuf::new().join("pgextworkdir"))
}

/// Install all extensions
pub fn cmd_install_all(cmd: CmdInstallAll) -> Result<()> {
  let db = load_plugin_db()?;
  let mut failed = vec![];
  for (idx, plugin) in db.plugins.iter().enumerate() {
    println!("{}/{}", idx + 1, db.plugins.len());
    if let Err(err) = cmd_install(CmdInstall {
      name: plugin.name.clone(),
      verbose: cmd.verbose,
    }) {
      println!(
        "{}: {} {}",
        style("Error").red().bold(),
        style(&plugin.name).bold(),
        err
      );
      failed.push(plugin.name.clone());
    }
  }
  if !failed.is_empty() {
    println!("{}: {}", style("Failed to install").red().bold(), failed.join(", "));
  }
  Ok(())
}

/// Install an extension
pub fn cmd_install(cmd: CmdInstall) -> Result<()> {
  let db = load_plugin_db()?;
  let workdir = create_workdir()?;
  let workspace_config = load_workspace_config()?;
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
      crate::download::download_zip(url.to_string(), &download_path, &build_dir, cmd.verbose)?;
    } else if let Some(url) = &plugin.download_url_tar {
      let download_path = workdir.join("downloads").join(format!("{}.tar.gz", name_tag));
      crate::download::download_tar(url.to_string(), &download_path, &build_dir, cmd.verbose)?;
    } else if let Some(src) = &plugin.download_git {
      crate::download::download_git(src, &build_dir, cmd.verbose)?;
    } else if let Some(src) = &plugin.copy_from_contrib {
      let contrib_dir = PathBuf::from(&workspace_config.pg_contrib).join(format!("{}/", src));
      if !contrib_dir.exists() {
        return Err(anyhow!("Could not find contrib source: {}", contrib_dir.display()));
      }
      cmd!("cp", "-a", contrib_dir, &build_dir)
        .run()
        .context("failed to copy")?;
    } else if let Some(true) = &plugin.no_download {
      println!("{} {}", style("Skipping Download").bold().blue(), name_tag);
    } else {
      return Err(anyhow!("No download url found in plugindb.toml"));
    }

    match plugin.resolver.as_str() {
      "pgxs" => crate::resolve_pgxs::resolve_build_pgxs(plugin, &build_dir, &workspace_config.pg_config, cmd.verbose)?,
      "pgrx" => {
        cmd!(
          "cargo",
          "pgrx",
          "install",
          "-c",
          &workspace_config.pg_config,
          "--release"
        )
        .dir(&build_dir)
        .run()
        .context("failed to install")?;
      }
      "pgsrctree" => {
        println!("{}: in Postgres source tree", style("Skipping Build").bold().blue());
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
    Err(anyhow!("Could not find extension {} in plugindb.toml", cmd.name))
  }
}

/// Install `pgx_show_hooks` and `pgx_trace_hooks`
pub fn cmd_install_hook() -> Result<()> {
  let config = load_workspace_config()?;

  println!("{} {}", style("Building").blue().bold(), style("pgx_show_hooks").bold());
  cmd!("cargo", "pgrx", "install", "-c", &config.pg_config)
    .dir("pgx_show_hooks")
    .run()?;

  println!(
    "{} {}",
    style("Building").blue().bold(),
    style("pgx_trace_hooks").bold()
  );
  cmd!("cargo", "pgrx", "install", "-c", &config.pg_config)
    .dir("pgx_trace_hooks")
    .run()?;

  println!("{} {}", style("Building").blue().bold(), style("pgextmgr").bold());
  cmd!("cargo", "pgrx", "install", "-c", &config.pg_config)
    .dir("pgextmgr")
    .run()?;

  cmd!("cargo", "pgrx", "start", "pg15").dir("pgx_show_hooks").run()?;

  println!(
    "{} {}",
    style("Installing").blue().bold(),
    style("pgx_show_hooks").bold()
  );
  let whoami = cmd!("whoami").read()?;
  let mut client = Client::connect(
    &format!("host=localhost dbname=postgres user={} port=28815", whoami.trim()),
    NoTls,
  )?;
  client.execute("DROP EXTENSION IF EXISTS pgx_show_hooks;", &[])?;
  client.execute("CREATE EXTENSION IF NOT EXISTS pgx_show_hooks;", &[])?;

  println!("{}: setting your user as a superuser", style("Configure").blue().bold(),);
  client.execute(&format!("ALTER USER {} WITH SUPERUSER;", whoami), &[])?;

  cmd!("cargo", "pgrx", "stop", "pg15").dir("pgx_show_hooks").run()?;

  println!(
    "{} {}",
    style("Installed").green().bold(),
    style("pgx_show_hooks").bold()
  );

  Ok(())
}
