#![allow(unsafe_code, unused_unsafe, unknown_lints)]

/// Registers a raw function to be called at binary exit time.
///
/// Corresponds to `atexit` in C.
///
/// Unsupported on Windows platforms: the platform's libc `atexit` tracks a set
/// of functions per module.
///
/// # Safety
///
/// Rust does not provide any safety guarantees about life-before-main or
/// life-after-main. Ordering of destructors is not guaranteed, nor that a
/// destructor will be called at all.
#[inline(always)]
#[cfg(not(windows))]
pub unsafe fn at_binary_exit(cb: extern "C" fn()) {
    unsafe {
        _run_atexit(cb);
    }
}

/// Registers a raw function to be called at library (libc calls this a DSO
/// or "dynamic shared object") exit time.
///
/// Corresponds to `__cxa_atexit` in C, though the exit function argument is
/// not available.
///
/// # Safety
///
/// Rust does not provide any safety guarantees about life-before-main or
/// life-after-main. Ordering of destructors is not guaranteed, nor that a
/// destructor will be called at all.
#[inline(always)]
pub unsafe fn at_module_exit(cb: extern "C" fn()) {
    unsafe {
        #[cfg(all(not(windows), not(target_family = "wasm")))]
        _run_cxa_atexit(cb);
        #[cfg(any(windows, target_family = "wasm"))]
        _run_atexit(cb);
    }
}

/// Use `at_module_exit` instead.
#[doc(hidden)]
#[deprecated(since = "0.11.0", note = "Use `at_module_exit` instead")]
#[inline(always)]
pub unsafe fn at_library_exit(cb: extern "C" fn()) {
    unsafe {
        at_module_exit(cb);
    }
}

/// Register a function to be called at libc exit time.
#[cfg(all(not(miri), not(target_family = "wasm")))]
#[inline(always)]
unsafe fn _run_atexit(cb: unsafe extern "C" fn()) {
    #[allow(missing_unsafe_on_extern)] // MSRV
    extern "C" {
        fn atexit(cb: unsafe extern "C" fn());
    }
    unsafe {
        atexit(cb);
    }
}

#[cfg(all(not(miri), target_family = "wasm"))]
#[inline(always)]
unsafe fn _run_atexit(cb: unsafe extern "C" fn()) {
    #[allow(missing_unsafe_on_extern)] // MSRV
    extern "C" {
        fn atexit(cb: unsafe extern "C" fn()) -> i32;
    }
    unsafe {
        atexit(cb);
    }
}

/// Register a function scoped to the current dynamic shared object.
#[cfg(all(not(miri), not(windows), not(target_family = "wasm")))]
#[inline(always)]
unsafe fn _run_cxa_atexit(cb: extern "C" fn()) {
    #[allow(missing_unsafe_on_extern)] // MSRV
    extern "C" {
        static __dso_handle: *const u8;
        fn __cxa_atexit(
            cb: /*unsafe*/ extern "C" fn(_: *const u8),
            arg: *const u8,
            dso_handle: *const u8,
        );
    }
    extern "C" fn exit_fn(fn_ptr: *const u8) {
        let f: fn() = unsafe { ::core::mem::transmute(fn_ptr) };
        f()
    }
    unsafe {
        __cxa_atexit(exit_fn, cb as _, __dso_handle);
    }
}

#[cfg(miri)]
#[inline(always)]
unsafe fn _run_atexit(_cb: extern "C" fn()) {
    // no-op on miri
}

#[cfg(miri)]
#[inline(always)]
unsafe fn _run_cxa_atexit(_cb: extern "C" fn()) {
    // no-op on miri
}
