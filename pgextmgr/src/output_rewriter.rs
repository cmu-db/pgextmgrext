use std::ffi::c_int;

use pgrx::pg_sys::{DestReceiver, QueryDesc, TupleDescData, TupleTableSlot};

use crate::api::OutputRewriter;

#[repr(C)]
struct OutputDest {
  pub recv: pgrx::pg_sys::DestReceiver,
  pub rewriters: Vec<&'static OutputRewriter>,
  pub rewriter_instances: Vec<*mut std::ffi::c_void>,
  pub original_dest: *mut pgrx::pg_sys::DestReceiver,
}

struct Context<'a> {
  slot: *mut TupleTableSlot,
  recv: &'a OutputDest,
  depth: usize,
}

impl OutputDest {
  fn new(rewriters: Vec<&'static OutputRewriter>, original_dest: *mut pgrx::pg_sys::DestReceiver) -> Self {
    Self {
      recv: pgrx::pg_sys::DestReceiver {
        receiveSlot: Some(Self::receive_slot),
        rStartup: Some(Self::startup),
        rShutdown: Some(Self::shutdown),
        rDestroy: Some(Self::destroy),
        mydest: OUTPUT_REWRITER_DEST,
      },
      rewriters,
      rewriter_instances: vec![],
      original_dest,
    }
  }

  unsafe extern "C" fn receive_slot(slot: *mut TupleTableSlot, recv: *mut DestReceiver) -> bool {
    let recv = &mut *(recv as *mut OutputDest);
    let mut ctx = Context { slot, recv, depth: 0 };
    Self::receive_slot_callback((&mut ctx) as *mut _ as *mut std::ffi::c_void)
  }

  unsafe extern "C" fn receive_slot_callback(ctx: *mut std::ffi::c_void) -> bool {
    let ctx = &*(ctx as *const Context);
    let mut next_ctx = Context {
      slot: ctx.slot,
      recv: ctx.recv,
      depth: ctx.depth + 1,
    };
    if ctx.depth < ctx.recv.rewriters.len() {
      let rr = ctx.recv.rewriter_instances[ctx.depth];
      ctx.recv.rewriters[ctx.depth].receive_slot.unwrap()(
        rr,
        ctx.slot,
        (&mut next_ctx) as *mut _ as *mut std::ffi::c_void,
        Self::receive_slot_callback,
      )
    } else {
      (*ctx.recv.original_dest).receiveSlot.unwrap()(ctx.slot, ctx.recv.original_dest)
    }
  }

  unsafe extern "C" fn destroy(recv: *mut DestReceiver) {
    let recv = &mut *(recv as *mut OutputDest);
    for (r, rr) in recv.rewriters.iter().zip(recv.rewriter_instances.iter()) {
      if let Some(f) = r.destroy {
        f(*rr);
      }
    }
    (*recv.original_dest).rDestroy.unwrap()(recv.original_dest)
  }

  unsafe extern "C" fn shutdown(recv: *mut DestReceiver) {
    let recv = &mut *(recv as *mut OutputDest);
    for (r, rr) in recv.rewriters.iter().zip(recv.rewriter_instances.iter()) {
      if let Some(f) = r.shutdown {
        f(*rr);
      }
    }
    (*recv.original_dest).rShutdown.unwrap()(recv.original_dest)
  }

  unsafe extern "C" fn startup(recv: *mut DestReceiver, operation: c_int, tuple_type: *mut TupleDescData) {
    let recv = &mut *(recv as *mut OutputDest);
    (*recv.original_dest).rStartup.unwrap()(recv.original_dest, operation, tuple_type);

    let mut rewriter_instances = vec![];
    for r in recv.rewriters.iter() {
      if let Some(f) = r.startup {
        rewriter_instances.push(f(operation, tuple_type));
      } else {
        rewriter_instances.push(std::ptr::null_mut());
      }
    }
    recv.rewriter_instances = rewriter_instances;
  }
}

pub(crate) unsafe extern "C" fn before_executor_run(query_desc: *mut QueryDesc, _: i32, _: u64, _: bool) {
  let mut rewriters: Vec<&'static OutputRewriter> = vec![];
  for (_, rewriter) in &crate::ALL_HOOKS.rewriters {
    if let Some(filter) = rewriter.filter {
      if !filter(query_desc) {
        continue;
      }
    }
    rewriters.push(rewriter);
  }
  if !rewriters.is_empty() {
    (*query_desc).dest =
      Box::leak(Box::new(OutputDest::new(rewriters, (*query_desc).dest))) as *mut _ as *mut DestReceiver;
  }
}

const OUTPUT_REWRITER_DEST: u32 = 2333;

pub(crate) unsafe extern "C" fn after_executor_run(query_desc: *mut QueryDesc, _: i32, _: u64, _: bool) {
  if let Some(dest) = (*query_desc).dest.as_mut() {
    if dest.mydest == OUTPUT_REWRITER_DEST {
      drop(Box::<OutputDest>::from_raw((*query_desc).dest as *mut OutputDest));
    }
  }
}
