use link_section::TypedMutableSection;

/// A collection of sized items that are available via sorted slice.
///
/// The gathered items are accessed via `&'static` references; the main section
/// is sorted by `T` before `main()`.
///
/// If the reference to the individual items is required, a sorted referenced
/// slice may be used instead.
pub struct ScatteredSortedSlice<T: Ord + 'static> {
    data: &'static TypedMutableSection<T>,
    _marker: core::marker::PhantomData<T>,
}

impl<T: Ord + 'static> ScatteredSortedSlice<T> {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub const unsafe fn new(data: &'static TypedMutableSection<T>) -> Self {
        Self {
            data,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T: Ord + 'static> ::core::ops::Deref for ScatteredSortedSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl<T: Ord + 'static> ::core::iter::IntoIterator for &'static ScatteredSortedSlice<T> {
    type Item = &'static T;
    type IntoIter = ::core::slice::Iter<'static, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.as_slice().into_iter()
    }
}

#[doc(hidden)]
pub unsafe fn initialize_scattered_sorted_slice<T: Ord>(main: &mut [T]) {
    let n = main.len();
    if n == 0 {
        return;
    }

    main.sort_unstable();
}

#[macro_export]
macro_rules! __sorted_slice {
    (@gather $(#[$meta:meta])* $vis:vis static $name:ident: ScatteredSortedSlice < $ty:ty >;) => {
        #[doc(hidden)]
        #[allow(unused)]
        $vis use $crate::__slice as $name;

        #[allow(unused)]
        #[allow(non_snake_case)]
        $vis mod $name {
            $crate::__support::link_section::declarative::section!(
                #[section(mutable)]
                pub static $name: $crate::__support::link_section::TypedMutableSection<$ty>;
            );
        }

        $(#[$meta])*
        $vis static $name: $crate::sorted_slice::ScatteredSortedSlice<$ty> = {
            unsafe { $crate::sorted_slice::ScatteredSortedSlice::new(
                self::$name::$name.const_deref()
            ) }
        };

        $crate::__support::ctor::declarative::ctor!(
            // Run as soon as possible, before any other constructors.
            #[ctor(unsafe, anonymous, priority = 0)]
            fn initialize_scattered_sorted_slice() {
                unsafe { $crate::sorted_slice::initialize_scattered_sorted_slice(self::$name::$name.as_mut_slice()) };
            }
        );
    };

    (@scatter [$($meta:tt)*] $(#[$imeta:meta])* $vis:vis $type:ident $name:tt: $ty:ty = $expr:expr;) => {
        $crate::__support::link_section::declarative::in_section!(
            #[in_section($($meta)*::$($meta)*)]
            $(#[$imeta])*
            $vis $type $name: $ty = $expr;
        );
    };
}

#[cfg(test)]
mod tests {
    __sorted_slice!(@gather pub static TEST_SORTED_SLICE: ScatteredSortedSlice<u32>;);
    __sorted_slice!(@scatter [TEST_SORTED_SLICE] pub const _: u32 = 6;);
    __sorted_slice!(@scatter [TEST_SORTED_SLICE] pub const _: u32 = 3;);
    __sorted_slice!(@scatter [TEST_SORTED_SLICE] pub const _: u32 = 2;);
    __sorted_slice!(@scatter [TEST_SORTED_SLICE] pub const _: u32 = 4;);
    __sorted_slice!(@scatter [TEST_SORTED_SLICE] pub const _: u32 = 5;);
    __sorted_slice!(@scatter [TEST_SORTED_SLICE] pub const _: u32 = 1;);

    #[test]
    fn test_scattered_sorted_slice() {
        assert_eq!(TEST_SORTED_SLICE.len(), 6);
        assert_eq!(&*TEST_SORTED_SLICE, [1, 2, 3, 4, 5, 6].as_slice());
        for item in &TEST_SORTED_SLICE {
            println!("item: {}", item);
        }
    }
}
