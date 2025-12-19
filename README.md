# Android-Mem-Kit

[![Crates.io](https://img.shields.io/crates/v/android-mem-kit.svg)](https://crates.io/crates/android-mem-kit)
[![Docs.rs](https://docs.rs/android-mem-kit/badge.svg)](https://docs.rs/android-mem-kit)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-android%20aarch64-green.svg)](https://developer.android.com/ndk)

**The Modern Standard Stack for Android Memory Instrumentation in Rust.**

`android-mem-kit` is a robust, type-safe wrapper around the battle-tested C/C++ libraries used in Android game modding and security research. It bridges the gap between low-level memory manipulation and Rust's safety, allowing you to write high-performance tools for rooted devices.

## Features

| Feature | Powered By | Description |
| :--- | :--- | :--- |
| **Hooking** | [Dobby](https://github.com/jmpews/Dobby) | Near-branch trampoline hooking support for ARM64/ARM. |
| **Patching** | [KittyMemory](https://github.com/MJx0/KittyMemory) | Runtime memory patching with hex string support & restore capability. |
| **Bypass** | [xdl](https://github.com/hexhacking/xdl) | Bypasses Android 7+ linker restrictions (dlopen/dlsym restrictions). |
| **Il2Cpp** | Built-in | Helper macros to resolve Il2Cpp exports dynamically without header files. |
| **Security** | [obfstr](https://crates.io/crates/obfstr) | Compile-time string obfuscation included by default. |

## Prerequisites

Since this crate compiles C++ code natively for Android, you need the **Android NDK**.

1.  **Install Android NDK** (r25b or newer recommended).
2.  **Set Environment Variable** before compiling:

    ```bash
    export ANDROID_NDK_HOME=/path/to/your/android-ndk-r29
    ```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
android-mem-kit = "0.1.0"
ctor = "0.2" # Recommended for library initialization
log = "0.4"
android_logger = "0.13"
```

## Quick Start

Here is a complete example of a mod menu backend or instrumentation tool.

```rust
use android_mem_kit::{
    hooking,
    memory::{self, MemoryPatch},
    il2cpp_call, // Macro for easy API calls
    obfstr::obfstr // String encryption
};
use ctor::ctor;
use std::ffi::c_void;

// 1. Setup Entry Point
#[ctor]
fn init() {
    android_logger::init_once(
        android_logger::Config::default().with_tag("MyMod"),
    );
    
    // Run in a separate thread to avoid blocking main thread
    std::thread::spawn(|| {
        log::info!("Library Loaded! Starting instrumentation...");
        start_mod();
    });
}

fn start_mod() {
    // 2. Locate Library Base
    // 'xdl' is used internally to bypass linker restrictions
    let lib_name = obfstr!("libil2cpp.so");
    let base = memory::get_lib_base(lib_name);
    
    if base == 0 {
        log::error!("Library not found!");
        return;
    }

    log::info!("{} base: {:#X}", lib_name, base);

    // 3. Memory Patching (Hex)
    // Example: Patching an integrity check at offset 0x123456
    let patch_offset = base + 0x123456;
    if let Ok(patch) = MemoryPatch::from_hex(patch_offset, "00 00 A0 E3") { // MOV R0, #0
        patch.apply();
        log::info!("Integrity check bypassed!");
        
        // You can restore it later:
        // patch.restore();
    }

    // 4. Function Hooking
    unsafe {
        let target_func = base + 0xABCDE;
        if let Ok(original) = hooking::attach(target_func, my_custom_hook as usize) {
            log::info!("Function hooked! Trampoline at: {:#X}", original);
        }
    }

    // 5. Il2Cpp API Call (Zero boilerplate)
    // Calls il2cpp_thread_attach(il2cpp_domain_get()) safely
    unsafe {
        let domain = il2cpp_call!("il2cpp_domain_get", usize, ).unwrap_or(0);
        if domain != 0 {
            il2cpp_call!("il2cpp_thread_attach", void, domain);
            log::info!("Attached to Il2Cpp thread!");
        }
    }
}

// Custom Hook Handler
unsafe extern "C" fn my_custom_hook(args: *mut c_void) {
    log::info!("Target function called with args: {:?}", args);
    // Call original function if needed...
}
```

## Building

Use `cargo-ndk` or standard cargo with target specification:

```bash
cargo build --target aarch64-linux-android --release
```

*Note: Ensure your `build.rs` can find the NDK toolchain.*

## Credits & Acknowledgements

This project wouldn't be possible without the open-source community. Huge thanks to the authors of the underlying C/C++ libraries:

*   **[Dobby](https://github.com/jmpews/Dobby)** by jmpews - Lightweight, multi-platform hooking framework.
*   **[KittyMemory](https://github.com/MJx0/KittyMemory)** by MJx0 - Memory manipulation library for Android/iOS.
*   **[xdl](https://github.com/hexhacking/xdl)** by hexhacking - The ultimate solution for Android linker restrictions.

## License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.