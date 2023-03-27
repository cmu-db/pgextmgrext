use std::io::{BufWriter, Write as _};
use std::time::Duration;

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use pgext_hook_macros::{for_all_hooks, for_all_plpgsql_hooks};
use postgres::Client;

use crate::cmd_install::create_workdir;
use crate::config::{edit_pgconf, load_workspace_config};
use crate::plugin::{find_plugin, load_plugin_db};
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
  let db = load_plugin_db()?;
  let config = load_workspace_config()?;
  let first = find_plugin(&db, &cmd.first)?;
  let second = find_plugin(&db, &cmd.second)?;

  let println = |msg: String| {
    if let Some(ref pbar) = pbar {
      pbar.println(msg);
    } else {
      println!("{}", msg);
    }
  };

  println(format!(
    "{} {} {} {}",
    style("Testing compatibility between").blue().bold(),
    style(&first.name).bold(),
    style("and").blue().bold(),
    style(&second.name).bold()
  ));

  pgx_stop_pg15()?;

  // Testing first,second
  println(format!(
    "{} {}, {}",
    style("Checking order").blue().bold(),
    style(&first.name).bold(),
    style(&second.name).bold()
  ));

  let shared_preloads = edit_pgconf(&db, &config, &[&first, &second])?;
  pgx_start_pg15()?;

  let mut client = Client::connect_test_db()?;
  client.show_preload_libraries(println)?;
  client.handle_installed(println)?;
  client.create_exn_if_absent("pgx_show_hooks")?;

  client.create_exns_for(&first)?;
  client.create_exns_for(&second)?;
  let hooks = client.show_hooks_all(println)?;
  client.drop_exns_for(&second, println)?;
  client.drop_exns_for(&first, println)?;

  if cmd.check {
    let name_tag = format!("{}@{}", second.name, second.version);
    let workdir = create_workdir()?;
    let build_dir = workdir.join("builds").join(name_tag);

    println!("{} {}", style("Regression Testing").bold().blue(), second.name);

    if let Err(err) = pgxs_installcheck(&second, Some((&first, &shared_preloads)), &build_dir, &config.pg_config) {
      println(format!("{err}"));
      println(format!(
        "{} - {}",
        style("Failed").bold().red(),
        style(&second.name).bold()
      ));
    }
  }
  pgx_stop_pg15()?;

  // Testing second,first
  println(format!(
    "{} {}{} {}",
    style("Checking order").blue().bold(),
    style(&second.name).bold(),
    style(",").blue().bold(),
    style(&first.name).bold()
  ));

  let shared_preloads = edit_pgconf(&db, &config, &[&second, &first])?;
  pgx_start_pg15()?;

  let mut client = Client::connect_test_db()?;
  client.show_preload_libraries(println)?;
  client.handle_installed(println)?;
  client.create_exn_if_absent("pgx_show_hooks")?;

  client.create_exns_for(&second)?;
  client.create_exns_for(&first)?;
  let hooks_rev = client.show_hooks_all(println)?;
  client.drop_exns_for(&first, println)?;
  client.drop_exns_for(&second, println)?;

  if cmd.check {
    let name_tag = format!("{}@{}", second.name, second.version);
    let workdir = create_workdir()?;
    let build_dir = workdir.join("builds").join(name_tag);

    println!("{} {}", style("Regression Testing").bold().blue(), second.name);

    if let Err(err) = pgxs_installcheck(&first, Some((&second, &shared_preloads)), &build_dir, &config.pg_config) {
      println(format!("{err}"));
      println(format!(
        "{} - {}",
        style("Failed").bold().red(),
        style(&first.name).bold()
      ));
    }
  }

  pgx_stop_pg15()?;
  debug_assert_eq!(&hooks, &hooks_rev);
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
  edit_pgconf(&db, &config, &[&plugin])?;
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
