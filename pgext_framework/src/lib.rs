#![allow(clippy::missing_safety_doc)]

use pgx::prelude::*;

pgx::pg_module_magic!();

static mut INSTALLED_PLUGINS: Vec<String> = Vec::new();
static mut PLANNER_HOOKS: Vec<(String, pgx::pg_sys::planner_hook_type)> = Vec::new();
static mut NEXT_HOOK_ID: usize = 0;
static mut CURRENT_PLANNER_HOOK: pgx::pg_sys::planner_hook_type = None;

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
  INSTALLED_PLUGINS.push(std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned());
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
  let (_, hook) = PLANNER_HOOKS[0];
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
  if let Some((_, hook)) = PLANNER_HOOKS.get(id + 1) {
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
fn all() -> SetOfIterator<'static, String> {
  SetOfIterator::new(unsafe { INSTALLED_PLUGINS.clone() }.into_iter())
}

#[pg_extern]
fn hooks() -> SetOfIterator<'static, String> {
  SetOfIterator::new(unsafe { PLANNER_HOOKS.clone().into_iter().map(|(name, _)| name) })
}
