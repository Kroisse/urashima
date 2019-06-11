use std::ffi::CStr;
use std::os::raw::c_char;

use guard::guard;

use urashima::{error::Error, runtime::Runtime};

pub struct NaruRuntime {
    inner: Runtime,
    last_error: Option<Error>,
}

#[no_mangle]
pub extern "C" fn naru_runtime_new() -> *mut NaruRuntime {
    let rt = NaruRuntime {
        inner: Runtime::new(),
        last_error: None,
    };
    Box::into_raw(Box::new(rt))
}

#[no_mangle]
pub unsafe extern "C" fn naru_runtime_delete(rt: *mut NaruRuntime) {
    if rt.is_null() {
        return;
    }
    let rt = Box::from_raw(rt);
    drop(rt);
}

#[no_mangle]
pub unsafe extern "C" fn naru_runtime_last_error(rt: *mut NaruRuntime) -> *const Error {
    guard!(let Some(rt) = rt.as_mut() else { return std::ptr::null(); });
    rt.last_error.as_ref().map(|e| {let e: *const _ = e; e}).unwrap_or_else(std::ptr::null)
}

#[no_mangle]
pub unsafe extern "C" fn naru_runtime_execute(rt: *mut NaruRuntime, path: *const c_char) {
    guard!(let Some(rt) = rt.as_mut() else { return; });
    let path = CStr::from_ptr(path);
    if let Err(e) = rt.inner.execute(&*path.to_string_lossy()) {
        rt.last_error = Some(e);
    }
}
