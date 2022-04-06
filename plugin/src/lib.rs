use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;
use std::ptr;

use nix_doc::get_function_docs;

/// Get the docs for a function in the given file path at the given file position and return it as
/// a C string pointer
#[no_mangle]
pub extern "C" fn nd_get_function_docs(
    filename: *const c_char,
    line: usize,
    col: usize,
) -> *const c_char {
    let fname = unsafe { CStr::from_ptr(filename) };
    fname
        .to_str()
        .ok()
        .and_then(|f| {
            panic::catch_unwind(|| get_function_docs(f, line, col))
                .map_err(|e| {
                    eprintln!("panic!! {:#?}", e);
                    e
                })
                .ok()
        })
        .flatten()
        .and_then(|s| CString::new(s).ok())
        .map(|s| s.into_raw() as *const c_char)
        .unwrap_or(ptr::null())
}

/// Call this to free a string from nd_get_function_docs
#[no_mangle]
pub extern "C" fn nd_free_string(s: *const c_char) {
    unsafe {
        // cast note: this cast is turning something that was cast to const
        // back to mut
        drop(CString::from_raw(s as *mut c_char));
    }
}
