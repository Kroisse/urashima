#![no_std]

use core::ptr;

use cstr_core::CStr;
use libc::c_char;

use guard::guard;

use urashima::{error::Error, runtime::Runtime};

pub struct NaruRuntime {
    inner: Runtime,
    last_error: Option<Error>,
}

#[no_mangle]
pub unsafe extern "C" fn naru_runtime_init(rt: *mut NaruRuntime) {
    if rt.is_null() { return; }
    rt.write(NaruRuntime {
        inner: Runtime::new(),
        last_error: None
    });
}

#[no_mangle]
pub unsafe extern "C" fn naru_runtime_dispose(rt: *mut NaruRuntime) {
    if rt.is_null() { return; }
    drop(rt.read())
}

#[no_mangle]
pub unsafe extern "C" fn naru_runtime_last_error(rt: *mut NaruRuntime) -> *const Error {
    guard!(let Some(rt) = rt.as_mut() else { return ptr::null(); });
    rt.last_error
        .as_ref()
        .map(|e| e as *const _)
        .unwrap_or_else(ptr::null)
}

#[no_mangle]
pub unsafe extern "C" fn naru_runtime_execute(rt: *mut NaruRuntime, path: *const c_char) {
    guard!(let Some(rt) = rt.as_mut() else { return; });
    let path = CStr::from_ptr(path);
    let path = match path.to_str() {
        Ok(v) => v,
        Err(_) => {
            panic!("todo");
        }
    };
    if let Err(e) = rt.inner.execute(path) {
        rt.last_error = Some(e);
    }
}
