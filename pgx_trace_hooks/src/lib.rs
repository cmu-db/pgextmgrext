use pgx::prelude::*;

pgx::pg_module_magic!();

static mut PREV_EXECUTOR_START_HOOK: pg_sys::ExecutorStart_hook_type = None;

#[pg_guard]
extern "C" fn executor_start_hook(query_desc: *mut pg_sys::QueryDesc, eflags: i32) {
  info!("ExecutorStart");
  unsafe {
    if let Some(prev_hook) = PREV_EXECUTOR_START_HOOK {
      prev_hook(query_desc, eflags);
    } else {
      pg_sys::standard_ExecutorStart(query_desc, eflags);
    }
  }
}

#[pg_guard]
pub extern "C" fn _PG_init() {
  unsafe {
    if !pg_sys::process_shared_preload_libraries_in_progress {
      error!("pgx_trace_hooks is not in shared_preload_libraries");
    }

    PREV_EXECUTOR_START_HOOK = pg_sys::ExecutorStart_hook;
    pg_sys::ExecutorStart_hook = Some(executor_start_hook);
  }
}

extension_sql!("LOAD 'pgx_trace_hooks';", name = "load_pgx_trace_hooks", requires = []);