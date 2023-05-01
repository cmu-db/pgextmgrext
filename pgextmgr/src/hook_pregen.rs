//! Pre-generated pseudo hooks

// TODO: use macros to reduce duplicated code

use std::ffi::c_int;

use pgext_hook_macros::*;
use pgrx::pg_sys::*;
use pgrx::prelude::*;

use crate::hook_ext::*;

macro_rules! build_hook_function {
  ([ $prefix:ident, $id:tt, $cb:ident, ($ret:ty) ] { $( $param:ident : $t:ty ,)* }) => {
    paste::paste! {
      #[pg_guard]
      unsafe extern "C" fn [< $prefix _ $id >](
        $( $param : $t ),*
      ) -> $ret {
        $cb($id, $( $param ),*)
      }
    }
  };
}

macro_rules! generate_hooks {
  ( [ $prefix:ident, $cb:ident, $global:ident, $hook_type:ty, $params:ident, $retty:tt ] $($id:tt),* ) => {
    $(
      $params! { [ $prefix, $id, $cb, $retty ] build_hook_function }
    )*
    pub static $global: &[$hook_type] = &[
      $(
        paste::paste! { Some([< $prefix _ $id >]) }
      ),*
    ];
  };
}

macro_rules! generate_copies {
  ([$($x:tt),*] $t:tt) => {
    $t! { [$($x),*] 1, 2, 3, 4, 5 }
  };
}

generate_copies! { [
  __pgext_planner_hook,
  pgext_planner_hook_cb,
  PREGENERATED_PLANNER_HOOKS,
  planner_hook_type,
  planner_hook_params,
  (*mut PlannedStmt)
] generate_hooks }

generate_copies! { [
  __pgext_executor_start_hook,
  pgext_executor_start_hook_cb,
  PREGENERATED_EXECUTOR_START_HOOKS,
  ExecutorStart_hook_type,
  executor_start_hook_params,
  (())
] generate_hooks }

generate_copies! { [
  __pgext_executor_run_hook,
  pgext_executor_run_hook_cb,
  PREGENERATED_EXECUTOR_RUN_HOOKS,
  ExecutorRun_hook_type,
  executor_run_hook_params,
  (())
] generate_hooks }

generate_copies! { [
  __pgext_executor_finish_hook,
  pgext_executor_finish_hook_cb,
  PREGENERATED_EXECUTOR_FINISH_HOOKS,
  ExecutorFinish_hook_type,
  executor_finish_hook_params,
  (())
] generate_hooks }

generate_copies! { [
  __pgext_executor_end_hook,
  pgext_executor_end_hook_cb,
  PREGENERATED_EXECUTOR_END_HOOKS,
  ExecutorEnd_hook_type,
  executor_end_hook_params,
  (())
] generate_hooks }
