macro_rules! __ref {
    ($ref:ident < $generic:ident >) => {
        unsafe impl<T: Sync> Sync for $ref<$generic> {}

        impl<T: ::core::fmt::Debug> ::core::fmt::Debug for $ref<$generic> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::ops::Deref::deref(self).fmt(f)
            }
        }

        impl<T: PartialEq> PartialEq for $ref<$generic> {
            fn eq(&self, other: &Self) -> bool {
                ::core::ops::Deref::deref(self) == ::core::ops::Deref::deref(other)
            }
        }

        impl<T: Eq> Eq for $ref<$generic> {}

        impl<T: PartialOrd> PartialOrd for $ref<$generic> {
            fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
                ::core::ops::Deref::deref(self).partial_cmp(::core::ops::Deref::deref(other))
            }
        }

        impl<T: Ord> Ord for $ref<$generic> {
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                ::core::ops::Deref::deref(self).cmp(::core::ops::Deref::deref(other))
            }
        }
    };
}

pub(crate) use __ref;
