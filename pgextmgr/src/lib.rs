#![allow(clippy::missing_safety_doc)]

use pgx::prelude::*;

pgx::pg_module_magic!();

static mut INSTALLED_PLUGINS: Vec<String> = Vec::new();
static mut PLANNER_HOOKS: Vec<(String, pgx::pg_sys::planner_hook_type)> = Vec::new();
static mut NEXT_HOOK_ID: usize = 0;
static mut CURRENT_PLANNER_HOOK: pgx::pg_sys::planner_hook_type = None;

const ENABLE_LOGGING: bool = true;

fn get_next_planner_hook() -> pgx::pg_sys::planner_hook_type {
  unsafe {
    let id = NEXT_HOOK_ID;
    NEXT_HOOK_ID += 1;
    PREGENERATED_PLANNER_HOOKS[id]
  }
}

#[pg_guard]
#[no_mangle]
unsafe extern "C" fn __pgext_before_init(name: *const pgx::ffi::c_char) {
  let plugin_name = std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
  INSTALLED_PLUGINS.push(plugin_name.clone());
  let hook = get_next_planner_hook();
  CURRENT_PLANNER_HOOK = hook;
  pgx::pg_sys::planner_hook = hook;
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C" fn __pgext_after_init() {
  if pgx::pg_sys::planner_hook != CURRENT_PLANNER_HOOK {
    PLANNER_HOOKS.push((INSTALLED_PLUGINS.last().unwrap().clone(), pgx::pg_sys::planner_hook));
    pgx::pg_sys::planner_hook = Some(pgext_planner_hook);
  } else {
    NEXT_HOOK_ID -= 1;
  }
}

/// Postgres will directly call this hook.
#[pg_guard]
pub unsafe extern "C" fn pgext_planner_hook(
  parse: *mut pgx::pg_sys::Query,
  query_string: *const ::std::os::raw::c_char,
  cursor_options: ::std::os::raw::c_int,
  bound_params: pgx::pg_sys::ParamListInfo,
) -> *mut pgx::pg_sys::PlannedStmt {
  let (ref name, hook) = PLANNER_HOOKS[0];
  if ENABLE_LOGGING {
    info!("pgext_planner_hook: {}", name);
  }
  hook.unwrap()(parse, query_string, cursor_options, bound_params)
}

/// All extensions will call this hook after finishing their own work.
unsafe fn pgext_planner_hook_cb(
  id: usize,
  parse: *mut pgx::pg_sys::Query,
  query_string: *const ::std::os::raw::c_char,
  cursor_options: ::std::os::raw::c_int,
  bound_params: pgx::pg_sys::ParamListInfo,
) -> *mut pgx::pg_sys::PlannedStmt {
  if let Some((name, hook)) = PLANNER_HOOKS.get(id + 1) {
    if ENABLE_LOGGING {
      info!("pgext_planner_hook: {}", name);
    }
    // find the next extension in the saved planner hooks and call it
    hook.unwrap()(parse, query_string, cursor_options, bound_params)
  } else {
    // call the Postgres planner hook
    pgx::pg_sys::standard_planner(parse, query_string, cursor_options, bound_params)
  }
}

macro_rules! generate_planner_hook_copy {
  ( $($id:tt),* ) => {
    $(
      paste::paste! {
        #[pg_guard]
        unsafe extern "C" fn [< __pgext_planner_hook_ $id >](
          parse: *mut pgx::pg_sys::Query,
          query_string: *const ::std::os::raw::c_char,
          cursor_options: ::std::os::raw::c_int,
          bound_params: pgx::pg_sys::ParamListInfo,
        ) -> *mut pgx::pg_sys::PlannedStmt {
          pgext_planner_hook_cb($id, parse, query_string, cursor_options, bound_params)
        }
      }
    )*

    static PREGENERATED_PLANNER_HOOKS: &[pgx::pg_sys::planner_hook_type] = &[
      $(
        paste::paste! { Some([< __pgext_planner_hook_ $id >]) }
      ),*
    ];
  };
}

generate_planner_hook_copy! { 0, 1, 2, 3, 4, 5 }

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
  TableIterator::new(unsafe {
    PLANNER_HOOKS
      .clone()
      .into_iter()
      .enumerate()
      .map(|(id, (name, _))| ("planner_hook".to_string(), id as i64, name))
  })
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
pub mod tests {
  use pgx::prelude::*;

  #[pg_test]
  #[search_path(@extschema@)]
  fn test_plugin_install() -> Result<(), spi::Error> {
    Spi::run("LOAD 'pgext_test_plugin'")?;
    Spi::run("LOAD 'pgext_pg_stat_statements'")?;
    Spi::run("LOAD 'pgext_pg_hint_plan'")?;
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
          Some("pgext_test_plugin".to_string()),
          Some("pgext_pg_stat_statements".to_string()),
          Some("pgext_pg_hint_plan".to_string())
        ]
      );

      Ok::<_, pgx::spi::Error>(())
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
    vec![]
  }
}
