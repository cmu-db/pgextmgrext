#![allow(clippy::missing_safety_doc)]

pub mod api;
mod hook_ext;
mod hook_mgr;
mod hook_pregen;
mod output_rewriter;
mod pgext;

use std::collections::BTreeMap;

use hook_mgr::ALL_HOOKS;
use pgrx::pg_sys::AsPgCStr;
use pgrx::prelude::*;

pgrx::pg_module_magic!();

static mut INSTALLED_PLUGINS: Vec<String> = Vec::new();
static mut INSTALLED_PLUGINS_STATUS: BTreeMap<String, bool> = BTreeMap::new();
const ENABLE_LOGGING: bool = false;

#[pg_guard]
#[no_mangle]
pub unsafe extern "C" fn __pgext_before_init(name: *const pgrx::ffi::c_char) -> *mut api::PgExtApi {
  let plugin_name = std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
  INSTALLED_PLUGINS.push(plugin_name.clone());
  INSTALLED_PLUGINS_STATUS.insert(plugin_name.clone(), true);
  pgrx::pg_sys::planner_hook = ALL_HOOKS
    .planner_hook
    .before_register(Some(hook_ext::pgext_planner_hook), pgrx::pg_sys::planner_hook);
  pgrx::pg_sys::ExecutorRun_hook = ALL_HOOKS
    .executor_run_hook
    .before_register(Some(hook_ext::pgext_executor_run_hook), pgrx::pg_sys::ExecutorRun_hook);
  pgrx::pg_sys::ExecutorStart_hook = ALL_HOOKS.executor_start_hook.before_register(
    Some(hook_ext::pgext_executor_start_hook),
    pgrx::pg_sys::ExecutorStart_hook,
  );
  pgrx::pg_sys::ExecutorEnd_hook = ALL_HOOKS
    .executor_end_hook
    .before_register(Some(hook_ext::pgext_executor_end_hook), pgrx::pg_sys::ExecutorEnd_hook);
  pgrx::pg_sys::ExecutorFinish_hook = ALL_HOOKS.executor_finish_hook.before_register(
    Some(hook_ext::pgext_executor_finish_hook),
    pgrx::pg_sys::ExecutorFinish_hook,
  );
  Box::leak(Box::new(api::PgExtApi::new(plugin_name)))
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C" fn __pgext_after_init() {
  let p = INSTALLED_PLUGINS.last().unwrap().clone();
  pgrx::pg_sys::planner_hook = ALL_HOOKS
    .planner_hook
    .after_register(p.clone(), pgrx::pg_sys::planner_hook);
  pgrx::pg_sys::ExecutorStart_hook = ALL_HOOKS
    .executor_start_hook
    .after_register(p.clone(), pgrx::pg_sys::ExecutorStart_hook);
  pgrx::pg_sys::ExecutorRun_hook = ALL_HOOKS
    .executor_run_hook
    .after_register(p.clone(), pgrx::pg_sys::ExecutorRun_hook);
  pgrx::pg_sys::ExecutorFinish_hook = ALL_HOOKS
    .executor_finish_hook
    .after_register(p.clone(), pgrx::pg_sys::ExecutorFinish_hook);
  pgrx::pg_sys::ExecutorEnd_hook = ALL_HOOKS
    .executor_end_hook
    .after_register(p, pgrx::pg_sys::ExecutorEnd_hook);
}

#[pg_extern]
fn all() -> TableIterator<'static, (name!(order, i64), name!(plugin, String), name!(status, String))> {
  TableIterator::new(unsafe {
    INSTALLED_PLUGINS.clone().into_iter().enumerate().map(|(id, name)| {
      let enabled = {
        if let Some(&true) = INSTALLED_PLUGINS_STATUS.get(&name) {
          "enabled"
        } else {
          "disabled"
        }
      };
      (id as i64, name, enabled.to_string())
    })
  })
}

#[pg_guard]
fn change_status(extension: &str, status: bool) -> i64 {
  unsafe {
    if let Some(enabled) = INSTALLED_PLUGINS_STATUS.get_mut(extension) {
      *enabled = status;
      ALL_HOOKS.rewriters.iter_mut().for_each(|(name, _, enabled)| {
        if name == extension {
          *enabled = status;
        }
      });
      1
    } else {
      panic!("extension {} does not exist", extension)
    }
  }
}

#[pg_guard]
fn change_status_all(status: bool) -> i64 {
  unsafe {
    INSTALLED_PLUGINS_STATUS.iter_mut().for_each(|(_, enabled)| {
      *enabled = status;
    });
    ALL_HOOKS.rewriters.iter_mut().for_each(|(_, _, enabled)| {
      *enabled = status;
    });
    INSTALLED_PLUGINS_STATUS.len() as i64
  }
}

#[pg_extern]
fn enable(extension: &str) -> i64 {
  change_status(extension, true)
}

#[pg_extern]
fn enable_all() -> i64 {
  change_status_all(true)
}

#[pg_extern]
fn disable(extension: &str) -> i64 {
  change_status(extension, false)
}

#[pg_extern]
fn disable_all() -> i64 {
  change_status_all(false)
}

#[pg_extern]
fn hooks() -> TableIterator<'static, (name!(hook, String), name!(order, i64), name!(plugin, String))> {
  let mut data = vec![];
  unsafe {
    data.extend(
      ALL_HOOKS
        .planner_hook
        .hooks()
        .iter()
        .enumerate()
        .map(|(id, (name, _))| ("planner_hook".to_string(), id as i64, name.clone())),
    );
    data.extend(
      ALL_HOOKS
        .executor_start_hook
        .hooks()
        .iter()
        .enumerate()
        .map(|(id, (name, _))| ("executor_start_hook".to_string(), id as i64, name.clone())),
    );
    data.extend(
      ALL_HOOKS
        .executor_run_hook
        .hooks()
        .iter()
        .enumerate()
        .map(|(id, (name, _))| ("executor_run_hook".to_string(), id as i64, name.clone())),
    );
    data.extend(
      ALL_HOOKS
        .executor_finish_hook
        .hooks()
        .iter()
        .enumerate()
        .map(|(id, (name, _))| ("executor_finish_hook".to_string(), id as i64, name.clone())),
    );
    data.extend(
      ALL_HOOKS
        .executor_end_hook
        .hooks()
        .iter()
        .enumerate()
        .map(|(id, (name, _))| ("executor_end_hook".to_string(), id as i64, name.clone())),
    );
    data.extend(
      ALL_HOOKS
        .rewriters
        .iter()
        .enumerate()
        .map(|(id, (name, _, _))| ("pgext_rewriters".to_string(), id as i64, name.clone())),
    );
  }
  TableIterator::new(data.into_iter())
}

#[no_mangle]
unsafe extern "C" fn _PG_init() {
  __pgext_before_init("__pgext".as_pg_cstr());
  ALL_HOOKS.executor_run_hook.register(
    "__pgext".to_string(),
    Some(crate::pgext::before_executor_run),
    Some(crate::pgext::after_executor_run),
  );
  __pgext_after_init();
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
pub mod tests {
  use pgrx::{prelude::*, spi::SpiClient};

  #[pg_test]
  #[search_path(@extschema@)]
  fn test_plugin_install() -> Result<(), spi::Error> {
    Spi::run("CREATE EXTENSION pgext_pg_poop;")?;
    Spi::run("CREATE EXTENSION pgext_pg_stat_statements;")?;
    Spi::run("CREATE EXTENSION pgext_pg_hint_plan;")?;

    Spi::connect(|client| {
      let table = client.select("SELECT * FROM pgextmgr.all()", None, None)?;
      assert_eq!(table.columns()?, 3);
      assert_eq!(table.len(), 4);
      let plugins = table
        .into_iter()
        .map(|x| x.get_datum_by_name("plugin").unwrap().value::<String>().unwrap())
        .collect::<Vec<_>>();
      assert_eq!(
        plugins,
        vec![
          Some("__pgext".to_string()),
          Some("pgext_pg_stat_statements".to_string()),
          Some("pgext_pg_hint_plan".to_string()),
          Some("pgext_pg_poop".to_string()),
        ]
      );

      Ok::<_, pgrx::spi::Error>(())
    })?;

    Ok(())
  }

  /// Count the number of enabled plugins
  fn count_enabled_plugins(client: &SpiClient) -> Result<Option<i64>, spi::Error> {
    let table = client.select(
      "SELECT COUNT(*) FROM pgextmgr.all() where status = 'enabled'",
      None,
      None,
    )?;

    let enabled_count = table
      .into_iter()
      .next()
      .map(|x| x.get_datum_by_ordinal(1)?.value::<i64>())
      .unwrap();

    enabled_count
  }

  /// Gets the status of a plugin by name
  fn get_plugin_status(client: &SpiClient, plugin: &str) -> Result<Option<String>, spi::Error> {
    let query = format!("SELECT status FROM pgextmgr.all() where plugin = '{}'", plugin);
    let table = client.select(&query, None, None)?;
    let status = table
      .into_iter()
      .next()
      .map(|x| x.get_datum_by_ordinal(1)?.value::<String>())
      .unwrap();

    status
  }

  #[pg_test]
  #[search_path(@extschema@)]
  fn test_plugin_enable() -> Result<(), spi::Error> {
    Spi::run("CREATE EXTENSION pgext_pg_poop;")?;
    Spi::run("CREATE EXTENSION pgext_pg_stat_statements;")?;
    Spi::run("CREATE EXTENSION pgext_pg_hint_plan;")?;

    Spi::connect(|client| {
      // enabled all 4 extensions
      client.select("SELECT pgextmgr.enable_all()", None, None)?;

      let enabled_count = count_enabled_plugins(&client)?;
      assert_eq!(enabled_count, Some(4));

      // check status of pg_poop is enabled
      let status = get_plugin_status(&client, "pgext_pg_poop")?;
      assert_eq!(status, Some("enabled".to_string()));

      // disable pg_poop extension, number of enabled extension - 1
      client.select("SELECT pgextmgr.disable('pgext_pg_poop')", None, None)?;

      let status = get_plugin_status(&client, "pgext_pg_poop")?;
      assert_eq!(status, Some("disabled".to_string()));
      let enabled_count = count_enabled_plugins(&client)?;
      assert_eq!(enabled_count, Some(3));

      // disable all extensions
      client.select("SELECT pgextmgr.disable_all()", None, None)?;
      let enabled_count = count_enabled_plugins(&client)?;
      assert_eq!(enabled_count, Some(0));

      Ok::<_, pgrx::spi::Error>(())
    })?;

    Ok(())
  }
}

/// This module is required by `cargo pgx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
  pub fn setup(_options: Vec<&str>) {
    // perform one-off initialization when the pg_test framework starts
  }

  pub fn postgresql_conf_options() -> Vec<&'static str> {
    // return any postgresql.conf settings that are required for your tests
    vec!["shared_preload_libraries = 'pgextmgr,pgext_pg_stat_statements,pgext_pg_hint_plan,pgext_pg_poop'"]
  }
}
