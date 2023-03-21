use pgx::prelude::*;

pgx::pg_module_magic!();


static mut PREV_EXECUTOR_START_HOOK: pg_sys::ExecutorStart_hook_type = None;
static mut PREV_EXECUTOR_RUN_HOOK: pg_sys::ExecutorRun_hook_type = None;
static mut PREV_EXECUTOR_FINISH_HOOK: pg_sys::ExecutorFinish_hook_type = None;
static mut PREV_EXECUTOR_END_HOOK: pg_sys::ExecutorEnd_hook_type = None;
static mut PREV_PROCESS_UTILITY_HOOK: pg_sys::ProcessUtility_hook_type = None;

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
extern "C" fn executor_run_hook(
  query_desc: *mut pg_sys::QueryDesc,
  direction: pg_sys::ScanDirection,
  count: pg_sys::uint64,
  execute_once: bool,
) {
  info!("ExecutorRun");
  unsafe {
    if let Some(prev_hook) = PREV_EXECUTOR_RUN_HOOK {
      prev_hook(query_desc, direction, count, execute_once);
    } else {
      pg_sys::standard_ExecutorRun(query_desc, direction, count, execute_once);
    }
  }
}

#[pg_guard]
extern "C" fn executor_finish_hook(query_desc: *mut pg_sys::QueryDesc) {
  info!("ExecutorFinish");
  unsafe {
    if let Some(prev_hook) = PREV_EXECUTOR_FINISH_HOOK {
      prev_hook(query_desc);
    } else {
      pg_sys::standard_ExecutorFinish(query_desc);
    }
  }
}

#[pg_guard]
extern "C" fn executor_end_hook(query_desc: *mut pg_sys::QueryDesc) {
  info!("ExecutorEnd");
  unsafe {
    if let Some(prev_hook) = PREV_EXECUTOR_END_HOOK {
      prev_hook(query_desc);
    } else {
      pg_sys::standard_ExecutorEnd(query_desc);
    }
  }
}

#[pg_guard]
extern "C" fn process_utility_hook(
  pstmt: *mut pg_sys::PlannedStmt,
  query_string: *const std::os::raw::c_char,
  read_only_tree: bool,
  context: pg_sys::ProcessUtilityContext,
  params: pg_sys::ParamListInfo,
  query_env: *mut pg_sys::QueryEnvironment,
  dest: *mut pg_sys::DestReceiver,
  qc: *mut pg_sys::QueryCompletion,
) {
  info!("ProcessUtility");
  unsafe {
    if let Some(prev_hook) = PREV_PROCESS_UTILITY_HOOK {
      prev_hook(
        pstmt,
        query_string,
        read_only_tree,
        context,
        params,
        query_env,
        dest,
        qc,
      );
    } else {
      pg_sys::standard_ProcessUtility(
        pstmt,
        query_string,
        read_only_tree,
        context,
        params,
        query_env,
        dest,
        qc,
      );
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

    PREV_EXECUTOR_RUN_HOOK = pg_sys::ExecutorRun_hook;
    pg_sys::ExecutorRun_hook = Some(executor_run_hook);

    PREV_EXECUTOR_FINISH_HOOK = pg_sys::ExecutorFinish_hook;
    pg_sys::ExecutorFinish_hook = Some(executor_finish_hook);

    PREV_EXECUTOR_END_HOOK = pg_sys::ExecutorEnd_hook;
    pg_sys::ExecutorEnd_hook = Some(executor_end_hook);

    PREV_PROCESS_UTILITY_HOOK = pg_sys::ProcessUtility_hook;
    pg_sys::ProcessUtility_hook = Some(process_utility_hook);
  }
}

extension_sql!("LOAD 'pgx_trace_hooks';", name = "load_pgx_trace_hooks", requires = []);
