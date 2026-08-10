#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

/// Define an initializer function that runs before `main`. See [ctor][ctor].
///
/// [ctor]: crate::ctor::ctor
#[cfg(all(feature = "ctor", feature = "proc_macro"))]
#[doc(inline)]
pub use linktime_proc_macro::ctor_linktime as ctor;

#[cfg(all(feature = "ctor", target_os = "uefi"))]
#[doc(inline)]
pub use ctor::run_constructors;

#[cfg(all(feature = "dtor", feature = "proc_macro"))]
#[doc(inline)]
pub use linktime_proc_macro::dtor_linktime as dtor;

#[cfg(all(feature = "dtor", target_os = "uefi"))]
#[doc(inline)]
pub use dtor::run_destructors;

/// Typed and untyped link section support for Rust.
#[cfg(feature = "link-section")]
pub mod link_section {
    #[cfg(feature = "proc_macro")]
    #[doc(inline)]
    pub use linktime_proc_macro::section_linktime as section;

    #[cfg(feature = "proc_macro")]
    #[doc(inline)]
    pub use linktime_proc_macro::in_section_linktime as in_section;

    #[doc(inline)]
    pub use link_section::{
        MovableBackref, MovableRef, Ref, Section, TypedMovableSection, TypedMutableSection,
        TypedReferenceSection, TypedSection,
    };

    /// Declarative `section` / `in_section` macros (no proc-macro feature).
    pub mod declarative {
        #[doc(inline)]
        pub use link_section::declarative::{in_section, section};
    }
}

/// Declarative forms of `linktime` macros.
pub mod declarative {
    #[cfg(feature = "ctor")]
    #[doc(inline)]
    pub use ctor::declarative::ctor;

    #[cfg(feature = "dtor")]
    #[doc(inline)]
    pub use dtor::declarative::dtor;
}

#[doc(hidden)]
pub mod __support {
    #[cfg(feature = "ctor")]
    pub use ctor::declarative::ctor as ctor_parse;
    #[cfg(feature = "dtor")]
    pub use dtor::declarative::dtor as dtor_parse;
    #[cfg(feature = "link-section")]
    pub use link_section::declarative::in_section as in_section_parse;
    #[cfg(feature = "link-section")]
    pub use link_section::declarative::section as section_parse;
}
