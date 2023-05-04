use pgrx::pg_sys::{uint64, QueryDesc, ScanDirection};

use crate::output_rewriter;

pub(crate) unsafe extern "C" fn before_executor_run(
  query_desc: *mut QueryDesc,
  direction: ScanDirection,
  count: uint64,
  execute_once: bool,
) {
  output_rewriter::before_executor_run(query_desc, direction, count, execute_once)
}

pub(crate) unsafe extern "C" fn after_executor_run(
  query_desc: *mut QueryDesc,
  direction: ScanDirection,
  count: uint64,
  execute_once: bool,
) {
  output_rewriter::after_executor_run(query_desc, direction, count, execute_once)
}
