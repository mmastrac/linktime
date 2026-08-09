//! Cross-crate link-section smoke test under fat LTO.
//!
//! Sections and shared types live in `copied-types`; item registrations are
//! split across `register-a` and `register-b`.

use copied_types::{IMMUTABLE_LINK_SECTION, MUT_LINK_SECTION, VALUES};

// Pull registrar object code into the link (no direct API use).
use register_a as _;
use register_b as _;

// DragonFly's pthread_key_create hands out duplicate keys until libthread
// initializes at first thread creation, so std TLS setup aborts with a
// KEY_SENTVAL assertion (rust-lang/rust#112180). Spawn and join one no-op
// thread ahead of everything else. Raw libc, std::thread touches the broken
// TLS machinery itself.
#[cfg(target_os = "dragonfly")]
mod dragonfly_thread_init {
    use ctor::ctor;

    extern "C" fn noop(_: *mut libc::c_void) -> *mut libc::c_void {
        core::ptr::null_mut()
    }

    #[ctor(unsafe, priority = 0)]
    unsafe fn init_libthread() {
        unsafe {
            let mut thread: libc::pthread_t = core::mem::zeroed();
            if libc::pthread_create(
                &mut thread,
                core::ptr::null(),
                noop,
                core::ptr::null_mut(),
            ) == 0
            {
                libc::pthread_join(thread, core::ptr::null_mut());
            }
        }
    }
}

pub fn main() {
    // LLVM was optimizing these copies into memsets
    let mut copied_section = MUT_LINK_SECTION.iter().copied().collect::<Vec<_>>();
    copied_section.sort();
    eprintln!("MUTABLE: {:?}", copied_section);

    let mut copied_section = IMMUTABLE_LINK_SECTION.iter().copied().collect::<Vec<_>>();
    copied_section.sort();
    eprintln!("IMMUTABLE: {:?}", copied_section);

    let mut v: Vec<&'static u64> = VALUES.iter().copied().collect();
    eprintln!("LEN={}", v.len());

    v.sort_unstable_by_key(|p| **p);
    let vals: Vec<u64> = v.iter().map(|p| **p).collect();
    eprintln!("VALS={vals:?}");

    assert_eq!(v.len(), 5, "expected 5 gathered pointers");
    assert_eq!(
        vals,
        vec![10, 20, 30, 40, 50],
        "gathered pointers must read back their real values, not zeros"
    );
    assert!(
        v.iter().all(|p| **p != 0),
        "NULL pointer in gathered slice (provenance miscompile)"
    );
    eprintln!("OK");
}
