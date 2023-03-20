use std::fmt::Write as _;
use std::io::{BufWriter, Write as _};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Result};
use console::style;
use duct::cmd;
use indicatif::{ProgressBar, ProgressStyle};
use pgext_hook_macros::{for_all_hooks, for_all_plpgsql_hooks};
use postgres::{Client, NoTls};

use crate::config::load_workspace_config;
use crate::plugin::{load_plugin_db, InstallStrategy};
use crate::{CmdTest, CmdTestAll};

pub fn cmd_test_all(cmd: CmdTestAll) -> Result<()> {
  let db = load_plugin_db()?;

  let all_hooks = {
    let mut all_hooks = vec![];
    macro_rules! push_hook {
        ($($hook:ident,)*) => {
            $(
              all_hooks.push(stringify!($hook).to_string());
            )*
        };
      }
    for_all_hooks! { push_hook }
    macro_rules! push_plpgsql_hook {
      ($($hook:ident,)*) => {
          $(
            all_hooks.push(format!("PLpgSQL_plugin->{}", stringify!($hook)));
          )*
      };
    }
    for_all_plpgsql_hooks! { push_plpgsql_hook }
    all_hooks
  };

  let mut file = if let Some(fpath) = cmd.dump_to {
    let file = std::fs::OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .truncate(true)
      .open(fpath)?;
    let mut writer = BufWriter::new(file);

    write!(writer, "plugin")?;
    for hook in &all_hooks {
      write!(writer, ", {}", hook)?;
    }
    writeln!(writer)?;

    Some(writer)
  } else {
    None
  };

  let pbar = ProgressBar::new(db.plugins.len() as u64).with_style(ProgressStyle::with_template(
    "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}",
  )?);

  pbar.enable_steady_tick(Duration::from_millis(100));

  for plugin in db.plugins {
    pbar.set_message(plugin.name.clone());
    match cmd_test(
      CmdTest {
        name: plugin.name.clone(),
      },
      Some(pbar.clone()),
    ) {
      Ok(r) => {
        if let Some(ref mut f) = file {
          write!(f, "{}", plugin.name)?;
          for hook in &all_hooks {
            if r.contains(hook) {
              write!(f, ",yes")?;
            } else {
              write!(f, ",")?;
            }
          }
          writeln!(f)?;
          f.flush()?;
        }
      }
      Err(e) => pbar.println(format!(
        "{}: {} {}",
        style("Error").red().bold(),
        style(&plugin.name).bold(),
        e
      )),
    }
    pbar.inc(1);
  }
  pbar.finish_with_message("Done");
  Ok(())
}

pub fn cmd_test(cmd: CmdTest, pbar: Option<ProgressBar>) -> Result<Vec<String>> {
  let db = load_plugin_db()?;
  let config = load_workspace_config()?;

  let plugin = if let Some(plugin) = db.plugins.iter().find(|x| x.name == cmd.name) {
    plugin.clone()
  } else {
    anyhow::bail!("Plugin not found");
  };

  let println = |msg| {
    if let Some(ref pbar) = pbar {
      pbar.println(msg);
    } else {
      println!("{}", msg);
    }
  };

  println(format!(
    "{} {}",
    style("Testing").blue().bold(),
    style(&plugin.name).bold()
  ));

  cmd!("cargo", "pgx", "stop", "pg15")
    .dir("pgx_show_hooks")
    .stderr_null()
    .stdout_null()
    .run()?;
  {
    let conf = PathBuf::from(&config.pg_data).join("postgresql.conf");
    let pgconf = std::fs::read_to_string(&conf)?;
    let mut new_pgconf = String::new();
    for line in pgconf.lines() {
      if line.starts_with("shared_preload_libraries = ") {
        if let InstallStrategy::Preload | InstallStrategy::PreloadInstall = plugin.install_strategy {
          writeln!(
            new_pgconf,
            "shared_preload_libraries = '{}' # modified by pgext",
            plugin.name
          )?;
        } else {
          writeln!(new_pgconf, "shared_preload_libraries = ''  # modified by pgext")?;
        }
      } else {
        writeln!(new_pgconf, "{}", line)?;
      }
    }
    std::fs::write(conf, new_pgconf)?;
  }

  cmd!("cargo", "pgx", "start", "pg15")
    .dir("pgx_show_hooks")
    .stderr_null()
    .stdout_null()
    .run()?;
  let whoami = cmd!("whoami").read()?;
  let mut client = Client::connect(
    &format!("host=localhost dbname=postgres user={} port=28815", whoami.trim()),
    NoTls,
  )?;
  let result = client.query_one("SHOW shared_preload_libraries;", &[])?;
  println(format!("shared_preload_libraries: {}", result.get::<_, String>(0)));

  let result = client.query("SELECT extname, extversion FROM pg_extension;", &[])?;
  for x in result.iter() {
    let name = x.get::<_, String>(0);
    let ver = x.get::<_, String>(1);
    if name == "plpgsql" {
      // it's fine to keep them
      println(format!("installed pg_extension: {name}@{ver}"));
    } else if name == "pgx_show_hooks" {
      println(format!("installed pg_extension: {name}@{ver}"));
      const REQUIRED_VER: &str = "0.0.3";
      if ver != REQUIRED_VER {
        bail!("require pgx_show_hooks@{REQUIRED_VER}, but found {ver}");
      }
    } else {
      println(format!("dropping pg_extension: {name}@{ver}"));
      client.execute(&format!("DROP EXTENSION {};", name), &[])?;
    }
  }

  client.execute("CREATE EXTENSION IF NOT EXISTS pgx_show_hooks;", &[])?;

  for extname in plugin.dependencies.iter() {
    client.execute(&format!("CREATE EXTENSION IF NOT EXISTS {};", extname), &[])?;
  }

  if let InstallStrategy::Install | InstallStrategy::PreloadInstall = plugin.install_strategy {
    client.execute(&format!("CREATE EXTENSION IF NOT EXISTS {};", plugin.name), &[])?;
  }

  let rows = client.query("SELECT * FROM show_hooks.all();", &[])?;

  let mut hooks = vec![];

  for x in rows.iter() {
    if x.get::<_, Option<String>>(1).is_some() {
      let hook_name = x.get::<_, String>(0);
      println(format!("{}: installed", hook_name));
      hooks.push(hook_name);
    }
  }

  if let InstallStrategy::Install | InstallStrategy::PreloadInstall = plugin.install_strategy {
    client.execute(&format!("DROP EXTENSION {};", plugin.name), &[])?;
  }

  for extname in plugin.dependencies.iter().rev() {
    client.execute(&format!("DROP EXTENSION {};", extname), &[])?;
  }

  cmd!("cargo", "pgx", "stop", "pg15")
    .dir("pgx_show_hooks")
    .stderr_null()
    .stdout_null()
    .run()?;

  Ok(hooks)
}
