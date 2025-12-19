use std::ffi::{CString, c_void};
use std::sync::OnceLock;

unsafe extern "C" {
    fn bridge_xdl_open(filename: *const i8) -> *mut c_void;
    fn bridge_xdl_sym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
}

// Wrapper agar pointer bisa Send/Sync
struct Il2CppHandle(*mut c_void);
unsafe impl Send for Il2CppHandle {}
unsafe impl Sync for Il2CppHandle {}

static IL2CPP_HANDLE: OnceLock<Il2CppHandle> = OnceLock::new();

fn get_handle() -> Option<*mut c_void> {
    // get_or_init mengembalikan referensi ke Il2CppHandle
    let handle_wrapper = IL2CPP_HANDLE.get_or_init(|| {
        unsafe {
            let name = CString::new("libil2cpp.so").unwrap();
            let handle = bridge_xdl_open(name.as_ptr() as *const i8);
            
            // simpan null tidak apa-apa di dalam wrapper
            if handle.is_null() {
                Il2CppHandle(std::ptr::null_mut())
            } else {
                Il2CppHandle(handle)
            }
        }
    });

    // Ambil nilai pointer dari wrapper
    let ptr = handle_wrapper.0;
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}

pub fn resolve_export(symbol_name: &str) -> Option<usize> {
    let handle = get_handle()?; // Jika None langsung return
    
    let c_sym = CString::new(symbol_name).ok()?;
    unsafe {
        let addr = bridge_xdl_sym(handle, c_sym.as_ptr() as *const i8);
        if addr.is_null() {
            None
        } else {
            Some(addr as usize)
        }
    }
}

#[macro_export]
macro_rules! il2cpp_call {
    ($func_name:expr, $ret:ty, $($arg:expr),*) => {
        {
            static CACHED_ADDR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let mut addr = CACHED_ADDR.load(std::sync::atomic::Ordering::Relaxed);
            
            if addr == 0 {
                if let Some(a) = $crate::il2cpp::resolve_export($func_name) {
                    addr = a;
                    CACHED_ADDR.store(addr, std::sync::atomic::Ordering::Relaxed);
                }
            }
            
            if addr != 0 {
                let func: extern "C" fn($($arg),*) -> $ret = std::mem::transmute(addr);
                Some(func($($arg),*))
            } else {
                None
            }
        }
    };
}