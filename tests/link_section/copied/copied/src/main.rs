//! Cross-crate link-section smoke test under fat LTO.
//!
//! Sections and shared types live in `copied-types`; item registrations are
//! split across `register-a` and `register-b`.

use copied_types::{IMMUTABLE_LINK_SECTION, MUT_LINK_SECTION, VALUES};

// Pull registrar object code into the link (no direct API use).
use register_a as _;
use register_b as _;

pub fn main() {
    // Keep libthread_xu in DT_NEEDED under fat LTO. Without it, std's startup
    // pthread_key_create resolves to libc's broken stubs and aborts with a
    // KEY_SENTVAL assertion (rust-lang/rust#112180).
    #[cfg(target_os = "dragonfly")]
    std::hint::black_box(libc::pthread_create as usize);

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
