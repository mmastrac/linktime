#![doc = include_str!("../README.md")]

#[cfg(all(feature = "ctor", feature = "proc_macro"))]
pub use linktime_proc_macro::ctor_linktime as ctor;

#[cfg(all(feature = "dtor", feature = "proc_macro"))]
pub use linktime_proc_macro::dtor_linktime as dtor;

#[cfg(all(feature = "link-section", feature = "proc_macro"))]
pub use linktime_proc_macro::section_linktime as section;

#[cfg(all(feature = "link-section", feature = "proc_macro"))]
pub use linktime_proc_macro::in_section_linktime as in_section;

pub mod declarative {
    pub use ctor::declarative::*;
    pub use dtor::declarative::*;
    pub use link_section::declarative::*;
}
