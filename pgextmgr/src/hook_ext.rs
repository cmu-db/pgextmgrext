//! The hooks installed by pgextmgrext

use std::ffi::c_int;

use pgrx::pg_sys::*;
use pgrx::prelude::*;

use crate::hook_mgr::ALL_HOOKS;
use crate::ENABLE_LOGGING;

/// Postgres will directly call this hook.
#[pg_guard]
pub unsafe extern "C" fn pgext_planner_hook(
  parse: *mut pgrx::pg_sys::Query,
  query_string: *const ::std::os::raw::c_char,
  cursor_options: ::std::os::raw::c_int,
  bound_params: pgrx::pg_sys::ParamListInfo,
) -> *mut pgrx::pg_sys::PlannedStmt {
  pgext_planner_hook_cb(0, parse, query_string, cursor_options, bound_params)
}

/// All extensions will call this hook after finishing their own work.
pub unsafe fn pgext_planner_hook_cb(
  id: usize,
  parse: *mut pgrx::pg_sys::Query,
  query_string: *const ::std::os::raw::c_char,
  cursor_options: ::std::os::raw::c_int,
  bound_params: pgrx::pg_sys::ParamListInfo,
) -> *mut pgrx::pg_sys::PlannedStmt {
  if let Some((name, hook)) = ALL_HOOKS.planner_hook.hooks().get(id) {
    if ENABLE_LOGGING {
      info!("pgext_planner_hook: {}", name);
    }
    // find the next extension in the saved planner hooks and call it
    hook.unwrap()(parse, query_string, cursor_options, bound_params)
  } else {
    // call the Postgres planner hook
    pgrx::pg_sys::standard_planner(parse, query_string, cursor_options, bound_params)
  }
}

/// Postgres will directly call this hook.
#[pg_guard]
pub unsafe extern "C" fn pgext_executor_start_hook(query_desc: *mut QueryDesc, eflags: c_int) {
  pgext_executor_start_hook_cb(0, query_desc, eflags)
}

/// All extensions will call this hook after finishing their own work.
pub unsafe fn pgext_executor_start_hook_cb(id: usize, query_desc: *mut QueryDesc, eflags: c_int) {
  if let Some((name, hook)) = ALL_HOOKS.executor_start_hook.hooks().get(id) {
    // find the next extension in the saved planner hooks and call it
    hook.unwrap()(query_desc, eflags)
  } else {
    // call the Postgres planner hook
    pgrx::pg_sys::standard_ExecutorStart(query_desc, eflags)
  }
}

/// All extensions will call this hook after finishing their own work.
pub unsafe fn pgext_executor_run_hook_cb(
  id: usize,
  query_desc: *mut QueryDesc,
  direction: ScanDirection,
  count: uint64,
  execute_once: bool,
) {
  if let Some((name, hook)) = ALL_HOOKS.executor_run_hook.hooks().get(id) {
    // find the next extension in the saved planner hooks and call it
    hook.unwrap()(query_desc, direction, count, execute_once)
  } else {
    // call the Postgres planner hook
    pgrx::pg_sys::standard_ExecutorRun(query_desc, direction, count, execute_once)
  }
}

/// Postgres will directly call this hook.
#[pg_guard]
pub unsafe extern "C" fn pgext_executor_run_hook(
  query_desc: *mut QueryDesc,
  direction: ScanDirection,
  count: uint64,
  execute_once: bool,
) {
  pgext_executor_run_hook_cb(0, query_desc, direction, count, execute_once)
}

/// All extensions will call this hook after finishing their own work.
pub unsafe fn pgext_executor_finish_hook_cb(id: usize, query_desc: *mut QueryDesc) {
  if let Some((name, hook)) = ALL_HOOKS.executor_finish_hook.hooks().get(id) {
    // find the next extension in the saved planner hooks and call it
    hook.unwrap()(query_desc)
  } else {
    // call the Postgres planner hook
    pgrx::pg_sys::standard_ExecutorFinish(query_desc)
  }
}

/// Postgres will directly call this hook.
#[pg_guard]
pub unsafe extern "C" fn pgext_executor_finish_hook(query_desc: *mut QueryDesc) {
  pgext_executor_finish_hook_cb(0, query_desc)
}

/// All extensions will call this hook after finishing their own work.
pub unsafe fn pgext_executor_end_hook_cb(id: usize, query_desc: *mut QueryDesc) {
  if let Some((name, hook)) = ALL_HOOKS.executor_end_hook.hooks().get(id) {
    // find the next extension in the saved planner hooks and call it
    hook.unwrap()(query_desc)
  } else {
    // call the Postgres planner hook
    pgrx::pg_sys::standard_ExecutorEnd(query_desc)
  }
}

/// Postgres will directly call this hook.
#[pg_guard]
pub unsafe extern "C" fn pgext_executor_end_hook(query_desc: *mut QueryDesc) {
  pgext_executor_end_hook_cb(0, query_desc)
}
