//! Item handling.

/// Element type for this section handle ([`crate::TypedSection`], etc.).
pub trait SectionItemType {
    /// Item type stored or referenced in the section.
    type Item;
}

#[diagnostic::on_unimplemented(message = "Incorrect section type for item")]
/// Typed section compatibility for item `T`.
pub trait SectionItemTyped<T> {
    /// Item representation for this `T`.
    type Item;
}

#[cfg(test)]
mod tests {
    use crate::item::SectionItemType;
    use core::marker::PhantomData;

    assert_type_eq!(<crate::TypedSection<u32> as SectionItemType>::Item, u32);
    assert_type_eq!(
        <crate::TypedSection<&'static u32> as SectionItemType>::Item,
        &'static u32
    );

    macro_rules! assert_type_eq {
        ($lhs:ty, $rhs:ty) => {
            const _: () = {
                struct __AssertTypeEq<T, U>(PhantomData<T>, PhantomData<U>);
                trait __AssertTypeEqT {
                    const CHECK: bool = true;
                }
                impl<T> __AssertTypeEqT for __AssertTypeEq<T, T> {}

                _ = <__AssertTypeEq<$lhs, $rhs> as __AssertTypeEqT>::CHECK;
            };
        };
    }
    pub(crate) use assert_type_eq;
}
