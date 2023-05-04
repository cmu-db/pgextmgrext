use pgrx::pg_sys::*;

use crate::api;

pub enum HookType<T> {
  /// Compatible mode, when extensions are using the original way of registering
  /// hooks.
  Compatible(T),
  /// PgExt mode, where extensions registers before and after hooks.
  PgExt(T, T),
}

pub struct HookMgr<P: Clone, T: Copy + Clone + PartialEq + Eq + 'static> {
  available_callbacks: &'static [T],
  hooks: Vec<(P, HookType<T>)>,
  next_hook_id: usize,
  registered: bool,
  prev_hook: Option<(T, T)>,
}

impl<P: Clone, T: Copy + Clone + PartialEq + Eq + 'static> HookMgr<P, T> {
  pub const fn new(available_callbacks: &'static [T]) -> Self {
    Self {
      available_callbacks,
      hooks: Vec::new(),
      next_hook_id: 0,
      registered: false,
      prev_hook: None,
    }
  }

  pub fn before_register(&mut self, override_with: T, prev_hook: T) -> T {
    if let Some(hook) = self.available_callbacks.get(self.next_hook_id) {
      self.registered = false;
      self.next_hook_id += 1;
      self.prev_hook = Some((override_with, prev_hook));
      *hook
    } else {
      panic!("too many extensions")
    }
  }

  pub fn after_register(&mut self, plugin: P, hook: T) -> T {
    let (override_with, prev_hook) = self.prev_hook.take().unwrap();
    if hook == self.available_callbacks[self.next_hook_id - 1] {
      if self.registered {
        return override_with;
      } else {
        // the extension is not using this hook
        self.next_hook_id -= 1;
        return prev_hook;
      }
    }
    assert!(!self.registered, "extension registered twice");
    self.hooks.push((plugin, HookType::Compatible(hook)));
    override_with
  }

  pub fn register(&mut self, plugin: P, before: T, after: T) {
    assert!(!self.registered, "extension registered twice");
    self.registered = true;
    self.hooks.push((plugin, HookType::PgExt(before, after)));
  }

  // TODO: support register compatible hook, should return a callback function
  // pub fn register_compatible(&mut self, plugin: P, hook: T) {
  //   assert!(!self.registered, "extension registered twice");
  //   self.registered = true;
  //   self.hooks.push((plugin, HookType::Compatible(hook)));
  // }

  pub fn hooks(&self) -> &[(P, HookType<T>)] {
    &self.hooks
  }
}

pub struct AllHooks {
  pub planner_hook: HookMgr<std::string::String, planner_hook_type>,
  pub executor_start_hook: HookMgr<std::string::String, ExecutorStart_hook_type>,
  pub executor_run_hook: HookMgr<std::string::String, ExecutorRun_hook_type>,
  pub executor_finish_hook: HookMgr<std::string::String, ExecutorFinish_hook_type>,
  pub executor_end_hook: HookMgr<std::string::String, ExecutorEnd_hook_type>,
  pub rewriters: Vec<(std::string::String, api::OutputRewriter, bool)>,
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
      rewriters: Vec::new(),
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
