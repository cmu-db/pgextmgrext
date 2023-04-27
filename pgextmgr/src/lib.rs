#![allow(clippy::missing_safety_doc)]

mod hook_ext;
mod hook_mgr;
mod hook_pregen;

use hook_mgr::ALL_HOOKS;
use pgrx::prelude::*;

pgrx::pg_module_magic!();

static mut INSTALLED_PLUGINS: Vec<String> = Vec::new();
pub const ENABLE_LOGGING: bool = true;

#[pg_guard]
#[no_mangle]
unsafe extern "C" fn __pgext_before_init(name: *const pgrx::ffi::c_char) {
  let plugin_name = std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
  INSTALLED_PLUGINS.push(plugin_name);
  pgrx::pg_sys::planner_hook = ALL_HOOKS.planner_hook.before_register();
  pgrx::pg_sys::ExecutorRun_hook = ALL_HOOKS.executor_run_hook.before_register();
  pgrx::pg_sys::ExecutorStart_hook = ALL_HOOKS.executor_start_hook.before_register();
  pgrx::pg_sys::ExecutorEnd_hook = ALL_HOOKS.executor_end_hook.before_register();
  pgrx::pg_sys::ExecutorFinish_hook = ALL_HOOKS.executor_finish_hook.before_register();
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C" fn __pgext_after_init() {
  let p = INSTALLED_PLUGINS.last().unwrap().clone();
  if ALL_HOOKS
    .planner_hook
    .after_register(p.clone(), pgrx::pg_sys::planner_hook)
  {
    pgrx::pg_sys::planner_hook = Some(hook_ext::pgext_planner_hook);
  } else {
    pgrx::pg_sys::planner_hook = None;
  }
  if ALL_HOOKS
    .executor_start_hook
    .after_register(p.clone(), pgrx::pg_sys::ExecutorStart_hook)
  {
    pgrx::pg_sys::ExecutorStart_hook = Some(hook_ext::pgext_executor_start_hook);
  } else {
    pgrx::pg_sys::ExecutorStart_hook = None;
  }
  if ALL_HOOKS
    .executor_run_hook
    .after_register(p.clone(), pgrx::pg_sys::ExecutorRun_hook)
  {
    pgrx::pg_sys::ExecutorRun_hook = Some(hook_ext::pgext_executor_run_hook);
  } else {
    pgrx::pg_sys::ExecutorRun_hook = None;
  }
  if ALL_HOOKS
    .executor_finish_hook
    .after_register(p.clone(), pgrx::pg_sys::ExecutorFinish_hook)
  {
    pgrx::pg_sys::ExecutorFinish_hook = Some(hook_ext::pgext_executor_finish_hook);
  } else {
    pgrx::pg_sys::ExecutorFinish_hook = None;
  }
  if ALL_HOOKS
    .executor_end_hook
    .after_register(p.clone(), pgrx::pg_sys::ExecutorEnd_hook)
  {
    pgrx::pg_sys::ExecutorEnd_hook = Some(hook_ext::pgext_executor_end_hook);
  } else {
    pgrx::pg_sys::ExecutorEnd_hook = None;
  }
}

#[pg_extern]
fn all() -> TableIterator<'static, (name!(order, i64), name!(plugin, String))> {
  TableIterator::new(unsafe {
    INSTALLED_PLUGINS
      .clone()
      .into_iter()
      .enumerate()
      .map(|(id, name)| (id as i64, name))
  })
}

#[pg_extern]
fn hooks() -> TableIterator<'static, (name!(hook, String), name!(order, i64), name!(plugin, String))> {
  let mut data = vec![];
  unsafe {
    data.extend(
      ALL_HOOKS
        .planner_hook
        .hooks()
        .to_vec()
        .into_iter()
        .enumerate()
        .map(|(id, (name, _))| ("planner_hook".to_string(), id as i64, name)),
    );
    data.extend(
      ALL_HOOKS
        .executor_start_hook
        .hooks()
        .to_vec()
        .into_iter()
        .enumerate()
        .map(|(id, (name, _))| ("executor_start_hook".to_string(), id as i64, name)),
    );
    data.extend(
      ALL_HOOKS
        .executor_run_hook
        .hooks()
        .to_vec()
        .into_iter()
        .enumerate()
        .map(|(id, (name, _))| ("executor_run_hook".to_string(), id as i64, name)),
    );
    data.extend(
      ALL_HOOKS
        .executor_finish_hook
        .hooks()
        .to_vec()
        .into_iter()
        .enumerate()
        .map(|(id, (name, _))| ("executor_finish_hook".to_string(), id as i64, name)),
    );
    data.extend(
      ALL_HOOKS
        .executor_end_hook
        .hooks()
        .to_vec()
        .into_iter()
        .enumerate()
        .map(|(id, (name, _))| ("executor_end_hook".to_string(), id as i64, name)),
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
    Spi::run("CREATE EXTENSION pgext_test_plugin;")?;
    Spi::run("CREATE EXTENSION pgext_pg_stat_statements;")?;
    Spi::run("CREATE EXTENSION pgext_pg_hint_plan;")?;

    Spi::connect(|client| {
      let table = client.select("SELECT * FROM pgextmgr.all()", None, None)?;
      assert_eq!(table.columns()?, 2);
      assert_eq!(table.len(), 3);
      let plugins = table
        .into_iter()
        .map(|x| x.get_datum_by_name("plugin").unwrap().value::<String>().unwrap())
        .collect::<Vec<_>>();
      assert_eq!(
        plugins,
        vec![
          Some("pgext_pg_stat_statements".to_string()),
          Some("pgext_pg_hint_plan".to_string()),
          Some("pgext_test_plugin".to_string()),
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
    vec!["shared_preload_libraries = 'pgextmgr,pgext_pg_stat_statements,pgext_pg_hint_plan,pgext_test_plugin'"]
  }
}
