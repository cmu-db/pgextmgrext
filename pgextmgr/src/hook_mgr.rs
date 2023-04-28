use pgrx::pg_sys::*;

pub struct HookMgr<P: Clone, T: Copy + Clone + PartialEq + Eq + 'static> {
  available_callbacks: &'static [T],
  hooks: Vec<(P, T)>,
  next_hook_id: usize,
}

impl<P: Clone, T: Copy + Clone + PartialEq + Eq + 'static> HookMgr<P, T> {
  pub const fn new(available_callbacks: &'static [T]) -> Self {
    Self {
      available_callbacks,
      hooks: Vec::new(),
      next_hook_id: 0,
    }
  }

  pub fn before_register(&mut self) -> T {
    if let Some(hook) = self.available_callbacks.get(self.next_hook_id) {
      self.next_hook_id += 1;
      *hook
    } else {
      panic!("too many extensions")
    }
  }

  pub fn after_register(&mut self, plugin: P, hook: T) -> bool {
    if hook == self.available_callbacks[self.next_hook_id - 1] {
      // the extension is not using this hook
      self.next_hook_id -= 1;
      return false;
    }
    self.hooks.push((plugin, hook));
    true
  }

  pub fn hooks(&self) -> &[(P, T)] {
    &self.hooks
  }
}

pub struct AllHooks {
  pub planner_hook: HookMgr<std::string::String, planner_hook_type>,
  pub executor_start_hook: HookMgr<std::string::String, ExecutorStart_hook_type>,
  pub executor_run_hook: HookMgr<std::string::String, ExecutorRun_hook_type>,
  pub executor_finish_hook: HookMgr<std::string::String, ExecutorFinish_hook_type>,
  pub executor_end_hook: HookMgr<std::string::String, ExecutorEnd_hook_type>,
}

impl AllHooks {
  pub const fn new(
    a: &'static [planner_hook_type],
    b: &'static [ExecutorStart_hook_type],
    c: &'static [ExecutorRun_hook_type],
    d: &'static [ExecutorFinish_hook_type],
    e: &'static [ExecutorEnd_hook_type],
  ) -> Self {
    Self {
      planner_hook: HookMgr::new(a),
      executor_start_hook: HookMgr::new(b),
      executor_run_hook: HookMgr::new(c),
      executor_finish_hook: HookMgr::new(d),
      executor_end_hook: HookMgr::new(e),
    }
  }
}

pub static mut ALL_HOOKS: AllHooks = AllHooks::new(
  crate::hook_pregen::PREGENERATED_PLANNER_HOOKS,
  crate::hook_pregen::PREGENERATED_EXECUTOR_START_HOOKS,
  crate::hook_pregen::PREGENERATED_EXECUTOR_RUN_HOOKS,
  crate::hook_pregen::PREGENERATED_EXECUTOR_FINISH_HOOKS,
  crate::hook_pregen::PREGENERATED_EXECUTOR_END_HOOKS,
);
