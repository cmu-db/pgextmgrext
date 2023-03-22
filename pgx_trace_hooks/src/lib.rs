use pgx::prelude::*;

pgx::pg_module_magic!();

static mut PREV_EXPLAIN_GET_INDEX_NAME_HOOK: pg_sys::explain_get_index_name_hook_type = None;
static mut PREV_EXPLAIN_ONE_QUERY_HOOK: pg_sys::ExplainOneQuery_hook_type = None;
static mut PREV_GET_ATTAVGWIDTH_HOOK: pg_sys::get_attavgwidth_hook_type = None;
static mut PREV_GET_INDEX_STATS_HOOK: pg_sys::get_index_stats_hook_type = None;
static mut PREV_GET_RELATION_INFO_HOOK: pg_sys::get_relation_info_hook_type = None;
static mut PREV_GET_RELATION_STATS_HOOK: pg_sys::get_relation_stats_hook_type = None;
static mut PREV_PLANNER_HOOK: pg_sys::planner_hook_type = None;
static mut PREV_JOIN_SEARCH_HOOK: pg_sys::join_search_hook_type = None;
static mut PREV_SET_REL_PATHLIST_HOOK: pg_sys::set_rel_pathlist_hook_type = None;
static mut PREV_SET_JOIN_PATHLIST_HOOK: pg_sys::set_join_pathlist_hook_type = None;
static mut PREV_CREATE_UPPER_PATHS_HOOK: pg_sys::create_upper_paths_hook_type = None;
static mut PREV_POST_PARSE_ANALYZE_HOOK: pg_sys::post_parse_analyze_hook_type = None;

#[pg_guard]
extern "C" fn explain_get_index_name_hook(index_id: pg_sys::Oid) -> *const std::os::raw::c_char {
  info!("explain_get_index_name:");
  unsafe {
    if let Some(prev_hook) = PREV_EXPLAIN_GET_INDEX_NAME_HOOK {
      return prev_hook(index_id);
    }
    std::ptr::null()
  }
}

extern "C" fn explain_one_query_hook(
  query: *mut pg_sys::Query,
  cursor_options: std::os::raw::c_int,
  into: *mut pg_sys::IntoClause,
  es: *mut pg_sys::ExplainState,
  query_string: *const std::os::raw::c_char,
  params: pg_sys::ParamListInfo,
  query_env: *mut pg_sys::QueryEnvironment,
) {
  info!("explain_one_query:");
  unsafe {
    if let Some(prev_hook) = PREV_EXPLAIN_ONE_QUERY_HOOK {
      prev_hook(query, cursor_options, into, es, query_string, params, query_env);
    }
    let bufusage_start = if (*es).buffers {
      Some(pg_sys::pgBufferUsage)
    } else {
      None
    };

    let bufusage = bufusage_start.and_then(|start| {
      let mut usage = pg_sys::BufferUsage::default();
      pg_sys::BufferUsageAccumDiff(&mut usage, &pg_sys::pgBufferUsage, &start);
      return Some(usage);
    });
    let plan = pg_sys::pg_plan_query(query, query_string, cursor_options, params);
    let plan_duration = pg_sys::instr_time::default();
    pg_sys::ExplainOnePlan(
      &mut *plan,
      &mut *into,
      &mut *es,
      &*query_string,
      params,
      &mut *query_env,
      &plan_duration,
      bufusage.map_or(std::ptr::null(), |usage| &usage),
    );

    // TODO(yuchen): seems no binding for pg_clock_gettime_ns in pgx
    // No standard_explain_one. Trying to time the plan duration (may be no need)
    // may need to do it manually or it doesn't matter
    // https://github.com/abigalekim/postgres/blob/order_hooks/src/backend/commands/explain.c#LL381C6-L381C26
    // https://github.com/abigalekim/postgres/blob/order_hooks/order_hooks/order_hooks.c
  }
}

#[pg_guard]
extern "C" fn get_attavgwidth_hook(relid: pg_sys::Oid, attnum: pg_sys::AttrNumber) -> pg_sys::int32 {
  info!("get_attavgwidth:");
  unsafe {
    if let Some(prev_hook) = PREV_GET_ATTAVGWIDTH_HOOK {
      return prev_hook(relid, attnum);
    }
    0
  }
}

#[pg_guard]
extern "C" fn get_index_stats_hook(
  root: *mut pg_sys::PlannerInfo,
  index_oid: pg_sys::Oid,
  indexattnum: pg_sys::AttrNumber,
  vardata: *mut pg_sys::VariableStatData,
) -> bool {
  info!("get_index_stats:");
  unsafe {
    if let Some(prev_hook) = PREV_GET_INDEX_STATS_HOOK {
      return prev_hook(root, index_oid, indexattnum, vardata);
    }
    false
  }
}

#[pg_guard]
extern "C" fn get_relation_info_hook(
  root: *mut pg_sys::PlannerInfo,
  relation_object_id: pg_sys::Oid,
  inhparent: bool,
  rel: *mut pg_sys::RelOptInfo,
) {
  info!("get_relation_info:");
  unsafe {
    if let Some(prev_hook) = PREV_GET_RELATION_INFO_HOOK {
      prev_hook(root, relation_object_id, inhparent, rel);
    }
  }
}

#[pg_guard]
extern "C" fn get_relation_stats_hook(
  root: *mut pg_sys::PlannerInfo,
  rte: *mut pg_sys::RangeTblEntry,
  attnum: pg_sys::AttrNumber,
  vardata: *mut pg_sys::VariableStatData,
) -> bool {
  info!("get_relation_stats:");
  unsafe {
    if let Some(prev_hook) = PREV_GET_RELATION_STATS_HOOK {
      return prev_hook(root, rte, attnum, vardata);
    }
    false
  }
}

#[pg_guard]
extern "C" fn planner_hook(
  parse: *mut pg_sys::Query,
  query_string: *const std::os::raw::c_char,
  cursor_options: std::os::raw::c_int,
  bound_params: pg_sys::ParamListInfo,
) -> *mut pg_sys::PlannedStmt {
  info!("planner:");
  unsafe {
    if let Some(prev_hook) = PREV_PLANNER_HOOK {
      return prev_hook(parse, query_string, cursor_options, bound_params);
    }
    pg_sys::standard_planner(parse, query_string, cursor_options, bound_params)
  }
}

#[pg_guard]
extern "C" fn join_search_hook(
  root: *mut pg_sys::PlannerInfo,
  levels_needed: std::os::raw::c_int,
  initial_rels: *mut pg_sys::List,
) -> *mut pg_sys::RelOptInfo {
  info!("join_search:");
  unsafe {
    if let Some(prev_hook) = PREV_JOIN_SEARCH_HOOK {
      return prev_hook(root, levels_needed, initial_rels);
    }
    pg_sys::standard_join_search(root, levels_needed, initial_rels)
  }
}

#[pg_guard]
extern "C" fn set_rel_pathlist_hook(
  root: *mut pg_sys::PlannerInfo,
  rel: *mut pg_sys::RelOptInfo,
  rti: pg_sys::Index,
  rte: *mut pg_sys::RangeTblEntry,
) {
  info!("set_rel_pathlist:");
  unsafe {
    if let Some(prev_hook) = PREV_SET_REL_PATHLIST_HOOK {
      prev_hook(root, rel, rti, rte);
    }
  }
}

#[pg_guard]
extern "C" fn set_join_pathlist_hook(
  root: *mut pg_sys::PlannerInfo,
  joinrel: *mut pg_sys::RelOptInfo,
  outerrel: *mut pg_sys::RelOptInfo,
  innerrel: *mut pg_sys::RelOptInfo,
  jointype: pg_sys::JoinType,
  extra: *mut pg_sys::JoinPathExtraData,
) {
  info!("set_join_pathlist:");
  unsafe {
    if let Some(prev_hook) = PREV_SET_JOIN_PATHLIST_HOOK {
      prev_hook(root, joinrel, outerrel, innerrel, jointype, extra);
    }
  }
}

#[pg_guard]
extern "C" fn create_upper_paths_hook(
  root: *mut pg_sys::PlannerInfo,
  stage: pg_sys::UpperRelationKind,
  input_rel: *mut pg_sys::RelOptInfo,
  output_rel: *mut pg_sys::RelOptInfo,
  extra: *mut std::os::raw::c_void,
) {
  info!("create_upper_paths:");
  unsafe {
    if let Some(prev_hook) = PREV_CREATE_UPPER_PATHS_HOOK {
      prev_hook(root, stage, input_rel, output_rel, extra);
    }
  }
}

#[pg_guard]
extern "C" fn post_parse_analyze_hook(
  pstate: *mut pg_sys::ParseState,
  query: *mut pg_sys::Query,
  jstate: *mut pg_sys::JumbleState,
) {
  info!("post_parse_analyze:");
  unsafe {
    if let Some(prev_hook) = PREV_POST_PARSE_ANALYZE_HOOK {
      prev_hook(pstate, query, jstate);
    }
  }
}

static mut PREV_EXECUTOR_START_HOOK: pg_sys::ExecutorStart_hook_type = None;
static mut PREV_EXECUTOR_RUN_HOOK: pg_sys::ExecutorRun_hook_type = None;
static mut PREV_EXECUTOR_FINISH_HOOK: pg_sys::ExecutorFinish_hook_type = None;
static mut PREV_EXECUTOR_END_HOOK: pg_sys::ExecutorEnd_hook_type = None;
static mut PREV_PROCESS_UTILITY_HOOK: pg_sys::ProcessUtility_hook_type = None;

#[pg_guard]
extern "C" fn executor_start_hook(query_desc: *mut pg_sys::QueryDesc, eflags: i32) {
  unsafe {
    info!("ExecutorStart:");
    // info!("query_desc={:#?}, eflags={}", *query_desc, eflags);
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
  unsafe {
    info!("ExecutorRun:");
    // info!("query_desc={:#?}, direction={}, count={}, execute_once={}", *query_desc, direction, count, execute_once);
    if let Some(prev_hook) = PREV_EXECUTOR_RUN_HOOK {
      prev_hook(query_desc, direction, count, execute_once);
    } else {
      pg_sys::standard_ExecutorRun(query_desc, direction, count, execute_once);
    }
  }
}

#[pg_guard]
extern "C" fn executor_finish_hook(query_desc: *mut pg_sys::QueryDesc) {
  unsafe {
    info!("ExecutorFinish:");
    // info!("query_desc={:#?}", *query_desc);
    if let Some(prev_hook) = PREV_EXECUTOR_FINISH_HOOK {
      prev_hook(query_desc);
    } else {
      pg_sys::standard_ExecutorFinish(query_desc);
    }
  }
}

#[pg_guard]
extern "C" fn executor_end_hook(query_desc: *mut pg_sys::QueryDesc) {
  info!("ExecutorEnd:");
  // info!("query_desc={:#?}", *query_desc);
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

    PREV_EXPLAIN_GET_INDEX_NAME_HOOK = pg_sys::explain_get_index_name_hook;
    pg_sys::explain_get_index_name_hook = Some(explain_get_index_name_hook);

    PREV_EXPLAIN_ONE_QUERY_HOOK = pg_sys::ExplainOneQuery_hook;
    pg_sys::ExplainOneQuery_hook = Some(explain_one_query_hook);

    PREV_GET_ATTAVGWIDTH_HOOK = pg_sys::get_attavgwidth_hook;
    pg_sys::get_attavgwidth_hook = Some(get_attavgwidth_hook);

    PREV_GET_INDEX_STATS_HOOK = pg_sys::get_index_stats_hook;
    pg_sys::get_index_stats_hook = Some(get_index_stats_hook);

    PREV_GET_RELATION_INFO_HOOK = pg_sys::get_relation_info_hook;
    pg_sys::get_relation_info_hook = Some(get_relation_info_hook);

    PREV_GET_RELATION_STATS_HOOK = pg_sys::get_relation_stats_hook;
    pg_sys::get_relation_stats_hook = Some(get_relation_stats_hook);

    PREV_PLANNER_HOOK = pg_sys::planner_hook;
    pg_sys::planner_hook = Some(planner_hook);

    PREV_JOIN_SEARCH_HOOK = pg_sys::join_search_hook;
    pg_sys::join_search_hook = Some(join_search_hook);

    PREV_SET_REL_PATHLIST_HOOK = pg_sys::set_rel_pathlist_hook;
    pg_sys::set_rel_pathlist_hook = Some(set_rel_pathlist_hook);

    PREV_SET_JOIN_PATHLIST_HOOK = pg_sys::set_join_pathlist_hook;
    pg_sys::set_join_pathlist_hook = Some(set_join_pathlist_hook);

    PREV_CREATE_UPPER_PATHS_HOOK = pg_sys::create_upper_paths_hook;
    pg_sys::create_upper_paths_hook = Some(create_upper_paths_hook);

    PREV_POST_PARSE_ANALYZE_HOOK = pg_sys::post_parse_analyze_hook;
    pg_sys::post_parse_analyze_hook = Some(post_parse_analyze_hook);

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
