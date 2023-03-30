use std::io::{BufWriter, Write as _};
use std::time::Duration;

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use pgext_hook_macros::{for_all_hooks, for_all_plpgsql_hooks};
use postgres::Client;

use crate::cmd_install::create_workdir;
use crate::config::{edit_pgconf, load_workspace_config};
use crate::plugin::{find_plugin, load_plugin_db, CheckStrategy, InstallStrategy};
use crate::resolve_pgxs::pgxs_installcheck;
use crate::test_control::{pgx_start_pg15, pgx_stop_pg15, ExtTestControl};
use crate::{CmdTest, CmdTestAll, CmdTestPair};

pub fn cmd_test_all(cmd: CmdTestAll) -> Result<()> {
  let db = load_plugin_db()?;
  let mut failed = vec![];

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
        check: cmd.check,
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
      Err(e) => {
        pbar.println(format!(
          "{}: {} {}",
          style("Error").red().bold(),
          style(&plugin.name).bold(),
          e
        ));
        failed.push(plugin.name.clone());
      }
    }
    pbar.inc(1);
  }
  pbar.finish_with_message("Done");

  if !failed.is_empty() {
    println!("{}: {}", style("Failed to test").red().bold(), failed.join(", "));
  }

  Ok(())
}

pub fn cmd_test_pair(cmd: CmdTestPair, pbar: Option<ProgressBar>) -> Result<Vec<String>> {
  let custom_sql = if cmd.run_custom_sql {
    Some(std::fs::read_to_string("custom.sql")?)
  } else {
    None
  };

  let db = load_plugin_db()?;
  let config = load_workspace_config()?;
  let mut plugins = vec![];
  for name in &cmd.exts {
    plugins.push(find_plugin(&db, name)?);
  }
  for plugin in &plugins {
    for dep in &plugin.dependencies {
      if !plugins.iter().any(|x| &x.name == dep) {
        println!("{}: dependency {} is required", style("Error").red().bold(), dep);
      }
    }
  }

  let println = |msg: String| {
    if let Some(ref pbar) = pbar {
      pbar.println(msg);
    } else {
      println!("{}", msg);
    }
  };

  println(format!(
    "{} {}",
    style("Installing").blue().bold(),
    plugins.iter().map(|x| style(&x.name).bold()).join(", ")
  ));

  pgx_stop_pg15()?;

  let shared_preloads = edit_pgconf(&db, &config, &plugins)?;
  pgx_start_pg15()?;

  let mut client = Client::connect_test_db()?;
  client.show_preload_libraries(println)?;
  client.handle_installed(println)?;
  client.create_exn_if_absent("pgx_show_hooks")?;

  for plugin in &plugins {
    match plugin.check_strategy {
      CheckStrategy::Install => client.create_exns_for(plugin)?,
      CheckStrategy::NoInstall => {
        println!("skipping create extension for {}", plugin.name);
      }
    }
  }
  let hooks = client.show_hooks_all(println)?;
  if custom_sql.is_some() {
    println!("{} with psql", style("Running custom.sql").blue().bold());
    // we want to see logs... and we have to use psql
    duct::cmd!(
      "psql",
      "-h",
      "localhost",
      "-d",
      "postgres",
      "-p",
      "28815",
      "-f",
      "custom.sql"
    )
    .run()?;
  }
  for plugin in plugins.iter().rev() {
    client.drop_exns_for(plugin, println)?;
  }

  if cmd.check {
    if let Some(second) = plugins.last() {
      let name_tag = format!("{}@{}", second.name, second.version);
      let workdir = create_workdir()?;
      let build_dir = workdir.join("builds").join(name_tag);

      println!("{} {}", style("Regression Testing").bold().blue(), second.name);
      let installs = plugins
        .iter()
        .filter_map(|x| match x.install_strategy {
          InstallStrategy::LoadInstall | InstallStrategy::PreloadInstall | InstallStrategy::Install => {
            Some(x.name.clone())
          }
          _ => None,
        })
        .take(plugins.len() - 1)
        .collect_vec();
      if let Err(err) = pgxs_installcheck(
        second,
        Some((&installs, &shared_preloads)),
        &build_dir,
        &config.pg_config,
      ) {
        println(format!("{err}"));
        println(format!(
          "{} - {}",
          style("Failed").bold().red(),
          style(&second.name).bold()
        ));
      }
    }
  }
  pgx_stop_pg15()?;

  Ok(hooks)
}

pub fn cmd_test(cmd: CmdTest, pbar: Option<ProgressBar>) -> Result<Vec<String>> {
  let db = load_plugin_db()?;
  let config = load_workspace_config()?;
  let plugin = find_plugin(&db, &cmd.name)?;

  let println = |msg: String| {
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

  pgx_stop_pg15()?;
  edit_pgconf(&db, &config, &[plugin.clone()])?;
  pgx_start_pg15()?;

  let mut client = Client::connect_test_db()?;
  client.show_preload_libraries(println)?;
  client.handle_installed(println)?;
  client.create_exn_if_absent("pgx_show_hooks")?;

  client.create_exns_for(&plugin)?;
  let hooks = client.show_hooks_all(println)?;
  client.drop_exns_for(&plugin, println)?;

  if cmd.check {
    let name_tag = format!("{}@{}", plugin.name, plugin.version);
    let workdir = create_workdir()?;
    let build_dir = workdir.join("builds").join(name_tag);

    println!("{} {}", style("Regression Testing").bold().blue(), plugin.name);
    if let Err(err) = pgxs_installcheck(&plugin, None, &build_dir, &config.pg_config) {
      println(format!("{err}"));
      println(format!(
        "{} - {}",
        style("Failed").bold().red(),
        style(&plugin.name).bold()
      ));
    }
  }

  pgx_stop_pg15()?;
  Ok(hooks)
}
