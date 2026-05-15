use link_section::TypedSection;

/// A collection of sized items that collected into a slice in an arbitrary
/// order.
pub struct ScatteredSlice<T: 'static> {
    section: &'static TypedSection<T>,
}

impl<T> ScatteredSlice<T> {
    pub const unsafe fn new(section: &'static TypedSection<T>) -> Self {
        Self { section }
    }
}

impl<T> ::core::ops::Deref for ScatteredSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.section
    }
}

#[macro_export]
macro_rules! __slice {
    (@gather $(#[$meta:meta])* $vis:vis static $name:ident: ScatteredSlice < $ty:ty >;) => {
        #[doc(hidden)]
        #[allow(unused)]
        $vis use $crate::__slice as $name;

        #[allow(unused)]
        #[allow(non_snake_case)]
        $vis mod $name {
            $crate::__support::link_section::declarative::section!(
                #[section(typed)]
                pub static $name: $crate::__support::link_section::TypedSection<$ty>;
            );
        }

        $(#[$meta])*
        $vis static $name: $crate::slice::ScatteredSlice<$ty> = {
            unsafe { $crate::slice::ScatteredSlice::new(
                self::$name::$name.const_deref()
            ) }
        };
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
    __slice!(@gather pub static TEST_SLICE: ScatteredSlice<u32>;);
    __slice!(@scatter [TEST_SLICE] pub const _: u32 = 1;);
    __slice!(@scatter [TEST_SLICE] pub const _: u32 = 3;);
    __slice!(@scatter [TEST_SLICE] pub const _: u32 = 2;);

    #[test]
    fn test_scattered_slice() {
        assert_eq!(TEST_SLICE.len(), 3);
        assert!(TEST_SLICE.contains(&1));
        assert!(TEST_SLICE.contains(&2));
        assert!(TEST_SLICE.contains(&3));
    }
}
