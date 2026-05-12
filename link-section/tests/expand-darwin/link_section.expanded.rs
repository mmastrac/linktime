use link_section::{section, in_section, TypedSection};
#[doc(hidden)]
pub use __FOO__link_section_private_macro__ as FOO;
#[allow(non_camel_case_types)]
struct FOO;
impl ::link_section::__support::SectionItemType for FOO {
    type Item = fn();
}
impl ::core::fmt::Debug for FOO {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::ops::Deref::deref(self).fmt(f)
    }
}
impl ::core::ops::Deref for FOO {
    type Target = TypedSection<fn()>;
    fn deref(&self) -> &Self::Target {
        self.const_deref()
    }
}
impl FOO {
    /// Get a `const` reference to the underlying section. In
    /// non-const contexts, `deref` is sufficient.
    pub const fn const_deref(&self) -> &TypedSection<fn()> {
        static SECTION: TypedSection<fn()> = {
            let section = {
                extern "C" {
                    #[link_name = "\u{1}section$start$__DATA$FOO"]
                    #[allow(unsafe_code)]
                    #[allow(unsafe_code)]
                    static __START: u8;
                }
                extern "C" {
                    #[link_name = "\u{1}section$end$__DATA$FOO"]
                    #[allow(unsafe_code)]
                    #[allow(unsafe_code)]
                    static __END: u8;
                }
                ::link_section::__support::PtrBounds::new(
                    unsafe { &raw const __START as *const () },
                    unsafe { &raw const __END as *const () },
                )
            };
            let name = "__DATA,FOO";
            unsafe { <TypedSection<fn()>>::new(name, section) }
        };
        &SECTION
    }
}
impl ::core::iter::IntoIterator for FOO {
    type Item = &'static fn();
    type IntoIter = ::core::slice::Iter<'static, fn()>;
    fn into_iter(self) -> Self::IntoIter {
        FOO.as_slice().iter()
    }
}
const _: () = {
    type __InSecStoredTy = <FOO as ::link_section::__support::SectionItemType>::Item;
    #[link_section = "__DATA,FOO,regular,no_dead_strip"]
    #[allow(unsafe_code)]
    #[used]
    static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = foo;
};
#[link_section = "__TEXT,FOO,regular,pure_instructions"]
#[allow(unsafe_code)]
#[allow(unsafe_code)]
fn foo() {
    {
        ::std::io::_print(format_args!("foo\n"));
    };
}
fn main() {}
