//! The hooks installed by pgextmgrext

use std::ffi::c_int;

use pgext_hook_macros::*;
use pgrx::pg_sys::*;
use pgrx::prelude::*;

use crate::hook_mgr::{HookType, ALL_HOOKS};
use crate::{ENABLE_LOGGING, INSTALLED_PLUGINS_STATUS};

macro_rules! build_hook_function {
  ([ $hook_func:ident, $cb_func:ident, $hook:ident, $standard_hook:ident, ($ret_ty:ty) ] { $( $param:ident : $t:ty ,)* }) => {
    /// Postgres will directly call this hook.
    #[pg_guard]
    pub unsafe extern "C" fn $hook_func(
      $( $param : $t ,)*
    ) -> $ret_ty {
      $cb_func(0, $( $param ),*)
    }

    /// All extensions will call this hook after finishing their own work.
    pub unsafe fn $cb_func(
      id: usize,
      $( $param : $t ,)*
    ) -> $ret_ty {
      if let Some((name, HookType::Compatible(hook))) = ALL_HOOKS.$hook.hooks().get(id) {
        if let Some(&true) = INSTALLED_PLUGINS_STATUS.get(name) {
          if ENABLE_LOGGING {
            info!("{}: {} (compatible)", stringify!($hook), name);
          }
          // find the next extension in the saved planner hooks and call it
          hook.unwrap()($( $param ),*)
        } else {
          // current hook disabled, skip
          $cb_func(id + 1, $( $param ),*)
        }
      } else if let Some((name, HookType::PgExt(before, after))) = ALL_HOOKS.$hook.hooks().get(id) {
        if let Some(&true) = INSTALLED_PLUGINS_STATUS.get(name) {
          if ENABLE_LOGGING {
            info!("{}: {} (pgext)", stringify!($hook), name);
          }
          // call the before hook
          before.unwrap()($( $param ),*);

          // call the next hook
          $cb_func(id + 1, $( $param ),*);

          // find the next extension in the saved planner hooks and call it
          after.unwrap()($( $param ),*)
        } else {
          // current hook disabled, skip
          $cb_func(id + 1, $( $param ),*)
        }
      } else {
        // call the Postgres planner hook
        pgrx::pg_sys::$standard_hook($( $param ),*)
      }
    }
  };
}

planner_hook_params! { [ pgext_planner_hook, pgext_planner_hook_cb, planner_hook, standard_planner, (*mut pgrx::pg_sys::PlannedStmt) ] build_hook_function }
executor_start_hook_params! { [ pgext_executor_start_hook, pgext_executor_start_hook_cb, executor_start_hook, standard_ExecutorStart, (()) ] build_hook_function }
executor_run_hook_params! { [ pgext_executor_run_hook, pgext_executor_run_hook_cb, executor_run_hook, standard_ExecutorRun, (()) ] build_hook_function }
executor_finish_hook_params! { [ pgext_executor_finish_hook, pgext_executor_finish_hook_cb, executor_finish_hook, standard_ExecutorFinish, (()) ] build_hook_function }
executor_end_hook_params! { [ pgext_executor_end_hook, pgext_executor_end_hook_cb, executor_end_hook, standard_ExecutorEnd, (()) ] build_hook_function }
