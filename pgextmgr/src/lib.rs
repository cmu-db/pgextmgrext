#![allow(clippy::missing_safety_doc)]

pub mod api;
mod hook_ext;
mod hook_mgr;
mod hook_pregen;
mod output_rewriter;

use std::collections::BTreeMap;

use hook_mgr::ALL_HOOKS;
use pgrx::prelude::*;

pgrx::pg_module_magic!();

static mut INSTALLED_PLUGINS: Vec<String> = Vec::new();
static mut INSTALLED_PLUGINS_STATUS: BTreeMap<String, bool> = BTreeMap::new();
const ENABLE_LOGGING: bool = true;

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
    INSTALLED_PLUGINS_STATUS
      .clone()
      .into_iter()
      .enumerate()
      .map(|(id, (name, enabled))| {
        (
          id as i64,
          name,
          (if enabled { "enabled" } else { "disabled" }).to_string(),
        )
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

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
pub mod tests {
  use pgrx::prelude::*;

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
