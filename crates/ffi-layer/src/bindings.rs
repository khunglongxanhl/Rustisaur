//! C bindings for Rustisaur.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use rustisaur_core::{EngineConfig, RustisaurEngine};
use tracing::debug;

use crate::wrappers::RustisaurHandle;

/// Create a new Rustisaur engine.
///
/// # Returns
///
/// A pointer to a new `RustisaurHandle`, or null if creation fails.
#[no_mangle]
pub extern "C" fn rustisaur_create() -> *mut RustisaurHandle {
    debug!("FFI: Creating new Rustisaur engine");
    match RustisaurEngine::new(EngineConfig::default()) {
        Ok(engine) => {
            debug!("FFI: Engine created successfully");
            Box::into_raw(Box::new(RustisaurHandle::new(engine)))
        }
        Err(e) => {
            debug!("FFI: Failed to create engine: {}", e);
            ptr::null_mut()
        }
    }
}

/// Destroy a Rustisaur engine.
///
/// # Safety
///
/// The `handle` pointer must be a valid pointer returned by `rustisaur_create`,
/// or null. It must not be used after this function is called.
#[no_mangle]
pub unsafe extern "C" fn rustisaur_destroy(handle: *mut RustisaurHandle) {
    if !handle.is_null() {
        debug!("FFI: Destroying engine at {:?}", handle);
        drop(Box::from_raw(handle));
    } else {
        debug!("FFI: Attempted to destroy null handle");
    }
}

/// Execute a script string. Returns result as C string (caller must free).
///
/// # Safety
///
/// The `handle` pointer must be a valid pointer returned by `rustisaur_create`.
/// The `script` pointer must be a valid null-terminated C string.
/// The returned string must be freed using `rustisaur_free_string`.
#[no_mangle]
pub unsafe extern "C" fn rustisaur_execute(
    handle: *mut RustisaurHandle,
    script: *const c_char,
) -> *mut c_char {
    if handle.is_null() || script.is_null() {
        debug!("FFI: Execute called with null pointer");
        return ptr::null_mut();
    }

    let handle = &*handle;
    let script_str = match CStr::from_ptr(script).to_str() {
        Ok(s) => {
            debug!("FFI: Executing script: {}", s);
            s
        }
        Err(e) => {
            debug!("FFI: Invalid UTF-8 in script: {}", e);
            return ptr::null_mut();
        }
    };

    match handle.engine().execute_script(script_str) {
        Ok(val) => {
            let result_str = format!("{val:?}");
            debug!("FFI: Script executed successfully: {}", result_str);
            CString::new(result_str)
                .map(|s| s.into_raw())
                .unwrap_or_else(|e| {
                    debug!("FFI: Failed to create CString: {}", e);
                    ptr::null_mut()
                })
        }
        Err(e) => {
            let error_str = format!("ERROR: {e}");
            debug!("FFI: Script execution failed: {}", error_str);
            CString::new(error_str)
                .map(|s| s.into_raw())
                .unwrap_or_else(|e| {
                    debug!("FFI: Failed to create error CString: {}", e);
                    ptr::null_mut()
                })
        }
    }
}

/// Free a C string returned by Rustisaur.
///
/// # Safety
///
/// The `s` pointer must be a valid pointer returned by `rustisaur_execute`,
/// or null. It must not be used after this function is called.
#[no_mangle]
pub unsafe extern "C" fn rustisaur_free_string(s: *mut c_char) {
    if !s.is_null() {
        debug!("FFI: Freeing string at {:?}", s);
        drop(CString::from_raw(s));
    } else {
        debug!("FFI: Attempted to free null string");
    }
}

/// Get Rustisaur version string.
#[no_mangle]
pub extern "C" fn rustisaur_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    debug!("FFI: Returning version: {}", env!("CARGO_PKG_VERSION"));
    VERSION.as_ptr() as *const c_char
}
