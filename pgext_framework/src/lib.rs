use pgx::prelude::*;

pgx::pg_module_magic!();

static mut INSTALLED_PLUGINS: Vec<String> = Vec::new();

#[pg_guard]
#[no_mangle]
pub extern "C" fn __pgext_before_init(name: *const pgx::ffi::c_char) {
  unsafe { INSTALLED_PLUGINS.push(std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned()) }
}

#[pg_guard]
#[no_mangle]
pub extern "C" fn __pgext_after_init() {}

#[pg_extern]
fn all() -> SetOfIterator<'static, String> {
  SetOfIterator::new(unsafe { INSTALLED_PLUGINS.clone() }.into_iter())
}
