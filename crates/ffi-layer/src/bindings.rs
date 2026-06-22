//! C bindings for Rustisaur.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use rustisaur_core::{EngineConfig, RustisaurEngine};

use crate::wrappers::RustisaurHandle;

/// Create a new Rustisaur engine.
#[no_mangle]
pub extern "C" fn rustisaur_create() -> *mut RustisaurHandle {
    match RustisaurEngine::new(EngineConfig::default()) {
        Ok(engine) => Box::into_raw(Box::new(RustisaurHandle::new(engine))),
        Err(_) => ptr::null_mut(),
    }
}

/// Destroy a Rustisaur engine.
#[no_mangle]
pub unsafe extern "C" fn rustisaur_destroy(handle: *mut RustisaurHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Execute a script string. Returns result as C string (caller must free).
#[no_mangle]
pub unsafe extern "C" fn rustisaur_execute(
    handle: *mut RustisaurHandle,
    script: *const c_char,
) -> *mut c_char {
    if handle.is_null() || script.is_null() {
        return ptr::null_mut();
    }

    let handle = &*handle;
    let script_str = match CStr::from_ptr(script).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match handle.engine().execute_script(script_str) {
        Ok(val) => CString::new(format!("{val:?}"))
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut()),
        Err(e) => CString::new(format!("ERROR: {e}"))
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut()),
    }
}

/// Free a C string returned by Rustisaur.
#[no_mangle]
pub unsafe extern "C" fn rustisaur_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Get Rustisaur version string.
#[no_mangle]
pub extern "C" fn rustisaur_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}
