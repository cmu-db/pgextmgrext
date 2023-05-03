use std::ffi::c_int;

use pgrx::pg_sys::{QueryDesc, TupleDesc, TupleTableSlot};

use crate::hook_mgr::ALL_HOOKS;

pub type OutputRewriterFilter = Option<extern "C" fn(query_desc: *mut QueryDesc) -> bool>;
pub type OutputRewriterStartup = Option<extern "C" fn(operation: c_int, type_info: TupleDesc) -> *mut std::ffi::c_void>;
pub type OutputRewriterShutdown = Option<extern "C" fn(*mut std::ffi::c_void)>;
pub type OutputRewriterDestroy = Option<extern "C" fn(*mut std::ffi::c_void)>;
pub type OutputRewriterReceiveSlot = Option<
  extern "C" fn(
    *mut std::ffi::c_void,
    slot: *mut TupleTableSlot,
    *mut std::ffi::c_void,
    unsafe extern "C" fn(*mut std::ffi::c_void) -> bool,
  ) -> bool,
>;

#[repr(C)]
#[derive(Clone)]
pub struct OutputRewriter {
  pub(crate) filter: OutputRewriterFilter,
  pub(crate) startup: OutputRewriterStartup,
  pub(crate) shutdown: OutputRewriterShutdown,
  pub(crate) destroy: OutputRewriterDestroy,
  pub(crate) receive_slot: OutputRewriterReceiveSlot,
}

#[repr(C)]
pub struct PgExtApi {
  plugin: *const String,
  register_output_rewriter: unsafe extern "C" fn(api: &PgExtApi, rewriter: &OutputRewriter),
}

impl PgExtApi {
  pub fn new(plugin: String) -> Self {
    PgExtApi {
      plugin: Box::leak(Box::new(plugin)),
      register_output_rewriter: Self::register_output_rewriter,
    }
  }

  unsafe extern "C" fn register_output_rewriter(api: &PgExtApi, rewriter: &OutputRewriter) {
    if !ALL_HOOKS
      .executor_run_hook
      .hooks()
      .iter()
      .any(|(name, _)| name == "__pgext")
    {
      ALL_HOOKS.executor_run_hook.register(
        "__pgext".to_string(),
        Some(crate::output_rewriter::before_executor_run),
        Some(crate::output_rewriter::after_executor_run),
      )
    }
    ALL_HOOKS.rewriters.push(((*api.plugin).clone(), rewriter.clone()));
  }
}
