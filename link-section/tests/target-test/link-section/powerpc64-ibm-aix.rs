use link_section::declarative::{section, in_section};
use link_section::TypedSection;




#[allow(non_camel_case_types)]
struct FOO;
#[doc(hidden)]
use ::link_section::__in_section_helper_macro_generic as FOO;
impl FOO {
    /// Get a `const` reference to the underlying section. In
    /// non-const contexts, `deref` is sufficient.
    pub const fn const_deref(&self) -> &'static TypedSection<fn()> {
        static SECTION: TypedSection<fn()> =
            {
                let section =
                    {
                        extern "C" {
                            #[link_name = "__start__data_link_section_FOO"]
                            #[allow(unsafe_code)]
                            static __START: u8;
                        }
                        extern "C" {
                            #[link_name = "__stop__data_link_section_FOO"]
                            #[allow(unsafe_code)]
                            static __END: u8;
                        }
                        ::link_section::__support::PtrBounds::new(unsafe {
                                &raw const __START as *const ()
                            }, unsafe { &raw const __END as *const () })
                    };
                let name = "_data_link_section_FOO";
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
    fn deref(&self) -> &Self::Target { self.const_deref() }
}
impl ::link_section::__support::SectionItemType for FOO {
    type Item = (fn());
}
impl ::link_section::__support::SectionItemTyped<(fn())> for FOO {
    type Item = (fn());
}
impl FOO {
    /// Get the section as a slice.
    pub fn as_slice(&self) -> &[(fn())] { self.const_deref().as_slice() }
}
impl ::core::iter::IntoIterator for FOO {
    type Item = &'static (fn());
    type IntoIter = ::core::slice::Iter<'static, (fn())>;
    fn into_iter(self) -> Self::IntoIter {
        self.const_deref().as_slice().iter()
    }
}
fn foo() {
    const _: () =
        {
            type __InSecStoredTy =
                <FOO as ::link_section::__support::SectionItemType>::Item;
            #[link_section = "_data_link_section_FOO"]
            #[used]
            #[export_name =
            "_expand_probe_expand_probe___LINK_SECTION_CONST_ITEM_L11C1"]
            static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = foo;
        };
}
fn main() {}
