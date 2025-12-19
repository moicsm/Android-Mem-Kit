use std::ffi::c_void;

unsafe extern "C" {
    fn DobbyHook(ptr: *mut c_void, handler: *mut c_void, original: *mut *mut c_void) -> i32;
}

/// Safe wrapper for hooking
pub unsafe fn attach(target: usize, replacement: usize) -> Result<usize, String> {
    let mut original_ptr: *mut c_void = std::ptr::null_mut();
    
    let ret = unsafe {
        DobbyHook(
            target as *mut c_void, 
            replacement as *mut c_void, 
            &mut original_ptr
        )
    };

    if ret == 0 {
        Ok(original_ptr as usize)
    } else {
        Err("Failed to hook function".to_string())
    }
}