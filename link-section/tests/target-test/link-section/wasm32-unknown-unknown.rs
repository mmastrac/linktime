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
                        static __LINK_SECTION_NAME: &'static str =
                            ".data.link_section.FOO";
                        #[export_name = ".data.link_section.FOO.bounds"]
                        #[used]
                        #[used]
                        static __LINK_SECTION_INFO:
                            ::link_section::__support::wasm::LinkSectionRawInfo =
                            ::link_section::__support::wasm::LinkSectionRawInfo::new::<(fn())>(__LINK_SECTION_NAME);
                        unsafe {
                            ::link_section::__support::Bounds::new(&raw const __LINK_SECTION_INFO)
                        }
                    };
                let name = ".data.link_section.FOO";
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
            #[link_section = ".data.link_section.FOO"]
            #[used]
            static __LINK_SECTION_CONST_ITEM: __InSecStoredTy = foo;
        };
}
fn main() {}
