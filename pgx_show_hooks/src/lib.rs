use pgext_hook_macros::{for_all_hooks, for_all_plpgsql_hooks};
use pgx::prelude::*;

pgx::pg_module_magic!();

fn render_addr<T>(func: Option<*const T>) -> Option<String> {
  func.and_then(|f| {
    if f.is_null() {
      None
    } else {
      Some(format!("{:16x}", f as usize))
    }
  })
}

#[pg_extern]
fn generate_series(start: i64, finish: i64, step: default!(i64, 1)) -> SetOfIterator<'static, i64> {
  SetOfIterator::new((start..=finish).step_by(step as usize))
}

#[pg_extern]
fn all() -> TableIterator<'static, (name!(name, String), name!(addr, Option<String>))> {
  let mut hooks = vec![];
  macro_rules! push_hook {
    ($($hook:ident,)*) => {
        $(
          hooks.push((
            stringify!($hook).to_string(),
            render_addr(unsafe { pg_sys::$hook }.map(|x| x as *const ())),
          ));
        )*
    };
  }
  for_all_hooks! { push_hook }

  unsafe {
    let name = std::ffi::CString::new("PLpgSQL_plugin").unwrap();
    let pgsql_plugin_ptr = pg_sys::find_rendezvous_variable(name.as_ptr()) as *const *const pg_sys::PLpgSQL_plugin;
    let pgsql_plugin_ptr = *pgsql_plugin_ptr;

    hooks.push((
      "PLpgSQL_plugin".to_string(),
      render_addr(Some(pgsql_plugin_ptr as *const _)),
    ));

    macro_rules! push_plpgsql_hook {
      ($($hook:ident,)*) => {
          $(
            hooks.push((
              format!("PLpgSQL_plugin->{}", stringify!($hook)),
              render_addr(
                pgsql_plugin_ptr
                  .as_ref()
                  .and_then(|x| x.$hook.map(|f| f as *const ())),
              ),
            ));
          )*
      };
    }

    for_all_plpgsql_hooks! { push_plpgsql_hook }
  }

  TableIterator::new(hooks.into_iter())
}
