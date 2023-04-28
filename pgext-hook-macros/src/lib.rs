#[macro_export]
macro_rules! for_all_hooks {
  ($macro:ident) => {
    $macro! {
      // General Hooks
      emit_log_hook,
      shmem_startup_hook,

      // Security Hooks
      check_password_hook,
      // ClientAuthentication_hook,
      ExecutorCheckPerms_hook,
      object_access_hook,
      row_security_policy_hook_permissive,
      row_security_policy_hook_restrictive,

      // Function Manager Hooks
      needs_fmgr_hook,
      fmgr_hook,

      // Planner Hooks
      explain_get_index_name_hook,
      ExplainOneQuery_hook,
      get_attavgwidth_hook,
      get_index_stats_hook,
      get_relation_info_hook,
      get_relation_stats_hook,
      planner_hook,
      join_search_hook,
      set_rel_pathlist_hook,
      set_join_pathlist_hook,
      create_upper_paths_hook,
      post_parse_analyze_hook,

      // Executor Hooks
      ExecutorStart_hook,
      ExecutorRun_hook,
      ExecutorFinish_hook,
      ExecutorEnd_hook,
      ProcessUtility_hook,
    }
  };
}

#[macro_export]
macro_rules! for_all_plpgsql_hooks {
  ($macro:ident) => {
    $macro! {
      func_setup,
      func_beg,
      func_end,
      stmt_beg,
      stmt_end,
    }
  };
}

#[macro_export]
macro_rules! planner_hook_params {
  ([ $($x:tt),* ] $macro:ident) => {
    $macro! {
      [ $($x),* ] {
        parse: *mut pgrx::pg_sys::Query,
        query_string: *const ::std::os::raw::c_char,
        cursor_options: ::std::os::raw::c_int,
        bound_params: pgrx::pg_sys::ParamListInfo,
      }
    }
  };
}

#[macro_export]
macro_rules! executor_start_hook_params {
  ([ $($x:tt),* ] $macro:ident) => {
    $macro! {
      [ $($x),* ] {
        query_desc: *mut QueryDesc,
        eflags: c_int,
      }
    }
  };
}

#[macro_export]
macro_rules! executor_run_hook_params {
  ([ $($x:tt),* ] $macro:ident) => {
    $macro! {
      [ $($x),* ] {
        query_desc: *mut QueryDesc,
        direction: ScanDirection,
        count: uint64,
        execute_once: bool,
      }
    }
  };
}

#[macro_export]
macro_rules! executor_finish_hook_params {
  ([ $($x:tt),* ] $macro:ident) => {
    $macro! {
      [ $($x),* ] {
        query_desc: *mut QueryDesc,
      }
    }
  };
}

#[macro_export]
macro_rules! executor_end_hook_params {
  ([ $($x:tt),* ] $macro:ident) => {
    $macro! {
      [ $($x),* ] {
        query_desc: *mut QueryDesc,
      }
    }
  };
}
