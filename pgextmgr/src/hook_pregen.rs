//! Pre-generated pseudo hooks

// TODO: use macros to reduce duplicated code

use std::ffi::c_int;

use pgrx::pg_sys::*;
use pgrx::prelude::*;

macro_rules! generate_planner_hook {
  ( $($id:tt),* ) => {
    $(
      paste::paste! {
        #[pg_guard]
        unsafe extern "C" fn [< __pgext_planner_hook_ $id >](
          parse: *mut Query,
          query_string: *const ::std::os::raw::c_char,
          cursor_options: ::std::os::raw::c_int,
          bound_params: ParamListInfo,
        ) -> *mut PlannedStmt {
          crate::hook_ext::pgext_planner_hook_cb($id, parse, query_string, cursor_options, bound_params)
        }
      }
    )*

    pub static PREGENERATED_PLANNER_HOOKS: &[planner_hook_type] = &[
      $(
        paste::paste! { Some([< __pgext_planner_hook_ $id >]) }
      ),*
    ];
  };
}

macro_rules! generate_executor_start_hook {
  ( $($id:tt),* ) => {
    $(
      paste::paste! {
        #[pg_guard]
        unsafe extern "C" fn [< __pgext_executor_start_hook_ $id >](
          query_desc: *mut QueryDesc, eflags: c_int,
        ) {
          crate::hook_ext::pgext_executor_start_hook_cb($id, query_desc, eflags)
        }
      }
    )*

    pub static PREGENERATED_EXECUTOR_START_HOOKS: &[ExecutorStart_hook_type] = &[
      $(
        paste::paste! { Some([< __pgext_executor_start_hook_ $id >]) }
      ),*
    ];
  };
}

macro_rules! generate_executor_run_hook {
  ( $($id:tt),* ) => {
    $(
      paste::paste! {
        #[pg_guard]
        unsafe extern "C" fn [< __pgext_executor_run_hook_ $id >](
          query_desc: *mut QueryDesc, direction: ScanDirection, count: uint64, execute_once: bool,
        ) {
          crate::hook_ext::pgext_executor_run_hook_cb($id, query_desc, direction, count, execute_once)
        }
      }
    )*

    pub static PREGENERATED_EXECUTOR_RUN_HOOKS: &[ExecutorRun_hook_type] = &[
      $(
        paste::paste! { Some([< __pgext_executor_run_hook_ $id >]) }
      ),*
    ];
  };
}

macro_rules! generate_executor_finish_hook {
  ( $($id:tt),* ) => {
    $(
      paste::paste! {
        #[pg_guard]
        unsafe extern "C" fn [< __pgext_executor_finish_hook_ $id >](
          query_desc: *mut QueryDesc,
        ) {
          crate::hook_ext::pgext_executor_finish_hook_cb($id, query_desc)
        }
      }
    )*

    pub static PREGENERATED_EXECUTOR_FINISH_HOOKS: &[ExecutorFinish_hook_type] = &[
      $(
        paste::paste! { Some([< __pgext_executor_finish_hook_ $id >]) }
      ),*
    ];
  };
}

macro_rules! generate_executor_end_hook {
  ( $($id:tt),* ) => {
    $(
      paste::paste! {
        #[pg_guard]
        unsafe extern "C" fn [< __pgext_executor_end_hook_ $id >](
          query_desc: *mut QueryDesc,
        ) {
          crate::hook_ext::pgext_executor_end_hook_cb($id, query_desc)
        }
      }
    )*

    pub static PREGENERATED_EXECUTOR_END_HOOKS: &[ExecutorEnd_hook_type] = &[
      $(
        paste::paste! { Some([< __pgext_executor_end_hook_ $id >]) }
      ),*
    ];
  };
}

macro_rules! generate_copies {
  ($t:tt) => {
    $t! { 1, 2, 3, 4, 5 }
  };
}

generate_copies! { generate_planner_hook }
generate_copies! { generate_executor_start_hook }
generate_copies! { generate_executor_run_hook }
generate_copies! { generate_executor_finish_hook }
generate_copies! { generate_executor_end_hook }
