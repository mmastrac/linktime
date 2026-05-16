use link_section::{section, in_section, TypedSection};
#[allow(non_camel_case_types)]
struct FOO;
#[doc(hidden)]
use ::link_section::__in_section_helper_macro_generic as FOO;
impl FOO {
    /// Get a `const` reference to the underlying section. In
    /// non-const contexts, `deref` is sufficient.
    pub const fn const_deref(&self) -> &'static TypedSection<fn()> {
        static SECTION: TypedSection<fn()> = {
            let section = {
                ::link_section::__support::PtrBounds::new(
                    {
                        #[allow(missing_unsafe_on_extern)]
                        extern "C" {
                            #[link_name = "\u{1}section$start$__DATA$FOO"]
                            static __SYMBOL: u8;
                        }
                        unsafe { &raw const __SYMBOL as *const () }
                    },
                    {
                        #[allow(missing_unsafe_on_extern)]
                        extern "C" {
                            #[link_name = "\u{1}section$end$__DATA$FOO"]
                            static __SYMBOL: u8;
                        }
                        unsafe { &raw const __SYMBOL as *const () }
                    },
                )
            };
            let name = "__DATA,FOO";
            ::link_section::__support::validate_section_name(name);
            unsafe { <TypedSection<fn()>>::new(name, section) }
        };
        &SECTION
    }
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
impl ::link_section::__support::SectionItemType for FOO {
    type Item = (fn());
}
impl ::link_section::__support::SectionItemTyped<(fn())> for FOO {
    type Item = (fn());
}
impl FOO {
    /// Get the section as a slice.
    pub fn as_slice(&self) -> &[(fn())] {
        self.const_deref().as_slice()
    }
}
impl ::core::iter::IntoIterator for FOO {
    type Item = &'static (fn());
    type IntoIter = ::core::slice::Iter<'static, (fn())>;
    fn into_iter(self) -> Self::IntoIter {
        self.const_deref().as_slice().iter()
    }
}
fn foo() {
    const _: fn() = const {
        type __InSecStoredTy = <FOO as ::link_section::__support::SectionItemType>::Item;
        const __LINK_SECTION_CONST_ITEM_VALUE: __InSecStoredTy = foo;
        #[used]
        #[link_section = "__DATA,FOO,regular,no_dead_strip"]
        static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = __LINK_SECTION_CONST_ITEM_VALUE;
        __LINK_SECTION_CONST_ITEM_VALUE
    };
    {
        ::std::io::_print(format_args!("foo\n"));
    };
}
fn main() {}
