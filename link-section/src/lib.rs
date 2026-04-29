#![doc = include_str!("../docs/BUILD.md")]
//! # link-section
#![doc = include_str!("../docs/PREAMBLE.md")]
#![allow(unsafe_code)]

#[doc = include_str!("../docs/LIFE_BEFORE_MAIN.md")]
pub mod life_before_main {}

/// Declarative forms of the `#[section]` and `#[in_section(...)]` macros.
///
/// The declarative forms wrap and parse a proc_macro-like syntax like so, and
/// are identical in expansion to the undecorated procedural macros. The
/// declarative forms support the same attribute parameters as the procedural
/// macros.
pub mod declarative {
    pub use crate::__in_section_parse as in_section;
    pub use crate::__section_parse as section;
}

#[doc(hidden)]
pub mod __support {
    pub use crate::__add_section_link_attribute as add_section_link_attribute;
    pub use crate::__def_section_name as def_section_name;
    pub use crate::__get_section as get_section;
    pub use crate::__in_section_crate as in_section_crate;
    pub use crate::__in_section_parse as in_section_parse;
    pub use crate::__section_parse as section_parse;

    #[cfg(feature = "proc_macro")]
    pub use link_section_proc_macro::hash;
    #[cfg(feature = "proc_macro")]
    pub use link_section_proc_macro::ident_concat;

    /// Declares the section_name macro.
    #[macro_export]
    #[doc(hidden)]
    macro_rules! __def_section_name {
        (
            {$(
                $__section:ident $__type:ident => $__prefix:tt __ $__suffix:tt;
            )*}
            AUXILIARY = $__aux_sep:literal;
            MAX_LENGTH = $__max_length:literal;
            HASH_LENGTH = $__hash_length:literal;
            VALID_SECTION_CHARS = $__valid_section_chars:literal;
        ) => {
            /// Internal macro for generating a section name.
            #[macro_export]
            #[doc(hidden)]
            macro_rules! __section_name {
                $(
                    (raw $__section $__type $name:ident) => {
                        concat!(concat! $__prefix, stringify!($name), concat! $__suffix);
                    };
                    (raw $__section $__type $name:ident $aux:ident) => {
                        concat!(concat! $__prefix, stringify!($name), $__aux_sep, stringify!($aux), concat! $__suffix);
                    };
                    ($pattern:tt $__section $__type $name:ident) => {
                        $crate::__support::hash!($pattern ($__prefix) $name ($__suffix) $__hash_length $__max_length $__valid_section_chars);
                    };
                    ($pattern:tt $__section $__type $name:ident $aux:ident) => {
                        $crate::__support::hash!($pattern ($__prefix) ($name $__aux_sep $aux) ($__suffix) $__hash_length $__max_length $__valid_section_chars);
                    };
                )*
                ($pattern:tt $unknown_section:ident $unknown_type:ident $name:ident) => {
                    const _: () = {
                        compile_error!("Unknown section type: `{}`/`{}`", stringify!($unknown_section), stringify!($unknown_type));
                    };
                };
            }
        };
    }

    #[cfg(feature = "proc_macro")]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __add_section_link_attribute(
        ($section:ident $type:ident $name:ident $($aux:ident)? #[$attr:ident = __] $item:item) => {
            $crate::__section_name!(
                (#[$attr = __] #[allow(unsafe_code)] $item)
                $section $type $name $($aux)?
            );
        }
    );

    #[cfg(not(feature = "proc_macro"))]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __add_section_link_attribute(
        ($section:ident $type:ident $name:ident #[$attr:ident = __] $item:item) => {
            #[$attr = $crate::__section_name!(
                raw $section $type $name
            )] $item
        }
    );

    // \x01: "do not mangle" (ref https://github.com/rust-lang/rust-bindgen/issues/2935)
    #[cfg(target_vendor = "apple")]
    def_section_name! {
        {
            data bare =>    ("__DATA,") __ ();
            code bare =>    ("__TEXT,") __ ();
            data section => ("__DATA,") __ (",regular,no_dead_strip");
            code section => ("__TEXT,") __ (",regular,no_dead_strip");
            data start =>   ("\x01section$start$__DATA$") __ ();
            data end =>     ("\x01section$end$__DATA$") __ ();
        }
        AUXILIARY = "_";
        MAX_LENGTH = 16;
        HASH_LENGTH = 6;
        VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    }

    #[cfg(target_family = "wasm")]
    def_section_name! {
        {
            data bare =>    (".data", ".link_section.") __ ();
            data section => (".data", ".link_section.") __ ();
            code bare =>    (".text", ".link_section.") __ ();
            code section => (".text", ".link_section.") __ ();
        }
        AUXILIARY = ".";
        MAX_LENGTH = 16;
        HASH_LENGTH = 6;
        VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    }

    #[cfg(all(
        not(target_vendor = "apple"),
        not(target_vendor = "pc"),
        not(target_os = "aix"),
        not(target_family = "wasm")
    ))]
    def_section_name! {
        {
            data bare =>    ("_data", "_link_section_") __ ();
            data section => ("_data", "_link_section_") __ ();
            data start =>   ("__start_", "_data", "_link_section_") __ ();
            data end =>     ("__stop_", "_data", "_link_section_") __ ();
            code bare =>    ("_text", "_link_section_") __ ();
            code section => ("_text", "_link_section_") __ ();
            code start =>   ("__start_", "_text", "_link_section_") __ ();
            code end =>     ("__stop_", "_text", "_link_section_") __ ();
        }
        AUXILIARY = "_";
        MAX_LENGTH = 64;
        HASH_LENGTH = 10;
        VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    }

    #[cfg(any(target_vendor = "pc"))]
    def_section_name! {
        {
            data bare =>    (".data", "$") __ ();
            data section => (".data", "$") __ ("$b");
            data start =>   (".data", "$") __ ("$a");
            data end =>     (".data", "$") __ ("$c");
            code bare =>    (".text", "$") __ ();
            code section => (".text", "$") __ ("$b");
            code start =>   (".text", "$") __ ("$a");
            code end =>     (".text", "$") __ ("$c");
        }
        AUXILIARY = "$d$";
        MAX_LENGTH = 64;
        HASH_LENGTH = 10;
        VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    }

    #[cfg(any(target_os = "aix"))]
    def_section_name! {
        {
            data bare =>    (".data", ".link_section.") __ ();
            data section => (".data", ".link_section.") __ (".b");
            data start =>   (".data", ".link_section.") __ (".a");
            data end =>     (".data", ".link_section.") __ (".c");
            code bare =>    (".text", ".link_section.") __ ();
            code section => (".text", ".link_section.") __ (".b");
            code start =>   (".text", ".link_section.") __ (".a");
            code end =>     (".text", ".link_section.") __ (".c");
        }
        AUXILIARY = ".d.";
        MAX_LENGTH = 64;
        HASH_LENGTH = 10;
        VALID_SECTION_CHARS = "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    }

    #[cfg(not(feature = "proc_macro"))]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __declare_macro {
        ($vis:vis $ident:ident $generic_macro:ident $args:tt) => {
            /// Internal macro for parsing the section. This is exported with
            /// the same name as the type below.
            #[doc(hidden)]
            $vis use $crate::$generic_macro as $ident;
        };
    }

    #[cfg(feature = "proc_macro")]
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __declare_macro {
        ($vis:vis $ident:ident $generic_macro:ident $args:tt) => {
            $crate::__support::ident_concat!(
                (#[macro_export]
                #[doc(hidden)]
                macro_rules!)  (__ $ident __link_section_private_macro__) ({
                    ($passthru:tt) => {
                        $crate::$generic_macro!($passthru $args);
                    };
                })
            );

            $crate::__support::ident_concat!(
                (#[doc(hidden)] pub use) (__ $ident __link_section_private_macro__) (as $ident;)
            );
        }
    }

    /// Define a link section.
    #[macro_export]
    #[doc(hidden)]
    macro_rules! __section_parse {
        // Has a generic (note that $generic eats the trailing semicolon)
        (#[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : $(:: $path_prefix:ident ::)? $($path:ident)::* < $($generic:tt)*) => {
            $crate::__section_parse!(@parsed #[section $($args)*] $(#[$meta])* $vis static $ident: ( $(:: $path_prefix ::)? $($path)::* < $($generic)*) ( $($generic)* ) __in_section_helper_macro_generic);
        };
        // No generic
        (#[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : $(:: $path_prefix:ident ::)? $($path:ident)::* ;) => {
            $crate::__section_parse!(@parsed #[section $($args)*] $(#[$meta])* $vis static $ident: ( $(:: $path_prefix ::)? $($path)::* ;) ( () > ; ) __in_section_helper_macro_no_generic);
        };
        // Both end up here...
        (@parsed #[section(aux = $name:ident, no_macro)] $(#[$meta:meta])* $vis:vis static $ident:ident : ($ty:ty ;) ( $generic_ty:ty > ; ) $generic_macro:ident) => {
            $crate::__section_parse!(@generate #[section(aux = $name)] $(#[$meta])* $vis static $ident: $ty, $generic_ty, $generic_macro);
        };
        (@parsed #[section(no_macro)] $(#[$meta:meta])* $vis:vis static $ident:ident : ($ty:ty ;) ( $generic_ty:ty > ; ) $generic_macro:ident) => {
            $crate::__section_parse!(@generate #[section] $(#[$meta])* $vis static $ident: $ty, $generic_ty, $generic_macro);
        };
        (@parsed #[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : ($ty:ty ;) ( $generic_ty:ty > ; ) $generic_macro:ident) => {
            $crate::__declare_macro!($vis $ident $generic_macro ($($args)*));
            $crate::__section_parse!(@generate #[section $($args)*] $(#[$meta])* $vis static $ident: $ty, $generic_ty, $generic_macro);
        };
        (@generate #[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : $ty:ty, $generic_ty:ty, __in_section_helper_macro_generic) => {
            $crate::__section_parse!(@generate #[section $($args)*] $(#[$meta])* $vis static $ident: $ty, $generic_ty, __base_case__);

            impl ::core::iter::IntoIterator for $ident {
                type Item = &'static $generic_ty;
                type IntoIter = ::core::slice::Iter<'static, $generic_ty>;
                fn into_iter(self) -> Self::IntoIter {
                    $ident.as_slice().iter()
                }
            }
        };
        (@generate #[section $($args:tt)*] $(#[$meta:meta])* $vis:vis static $ident:ident : $ty:ty, $generic_ty:ty, $generic_macro:ident) => {
            $(#[$meta])*
            #[allow(non_camel_case_types)]
            $vis struct $ident;

            impl $crate::__support::SectionItemType for $ident {
                type Item = $generic_ty;
            }

            impl ::core::fmt::Debug for $ident {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::ops::Deref::deref(self).fmt(f)
                }
            }

            impl ::core::ops::Deref for $ident {
                type Target = $ty;
                fn deref(&self) -> &Self::Target {
                    self.const_deref()
                }
            }

            $crate::__section_parse!(@deref #[section $($args)*] $(#[$meta])* $vis static $ident: $ty, $generic_ty, __base_case__);
        };
        (@deref #[section] $(#[$meta:meta])* $vis:vis static $ident:ident : $ty:ty, $generic_ty:ty, __base_case__) => {
            impl $ident {
                /// Get a `const` reference to the underlying section. In
                /// non-const contexts, `deref` is sufficient.
                pub const fn const_deref(&self) -> &$ty {
                    static SECTION: $ty = {
                        let section = $crate::__support::get_section!(name=$ident, type=$generic_ty, aux=);
                        let name = $crate::__section_name!(
                            raw data bare $ident
                        );
                        unsafe { <$ty>::new(name, section.0, section.1) }
                    };
                    &SECTION
                }
            }
        };
        (@deref #[section(aux=$aux:ident)] $(#[$meta:meta])* $vis:vis static $ident:ident : $ty:ty, $generic_ty:ty, __base_case__) => {
            impl $ident {
                /// Get a `const` reference to the underlying section. In
                /// non-const contexts, `deref` is sufficient.
                pub const fn const_deref(&self) -> &$ty {
                    static SECTION: $ty = {
                        let section = $crate::__support::get_section!(name=$ident, type=$generic_ty, aux=$aux);
                        let name = $crate::__section_name!(
                            raw data bare $aux $ident // swap
                        );
                        unsafe { <$ty>::new(name, section.0, section.1) }
                    };
                    &SECTION
                }
            }
        };
    }

    #[cfg(miri)]
    mod section {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __get_section {
            (name=$ident:ident, type=$generic_ty:ty, aux=$($aux:ident)?) => {{
                (std::ptr::null_mut(), std::ptr::null_mut())
            }};
        }

        /// On Apple platforms, the linker provides a pointer to the start and end
        /// of the section regardless of the section's name.
        pub type SectionPtr<T> = *const ::core::marker::PhantomData<T>;
    }

    #[cfg(all(not(miri), target_family = "wasm"))]
    mod section {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __get_section {
            (name=$ident:ident, type=$generic_ty:ty, aux=$($aux:ident)?) => {
                {
                    static __START: ::core::sync::atomic::AtomicPtr::<::core::marker::PhantomData<$generic_ty>> = unsafe {
                        ::core::sync::atomic::AtomicPtr::<::core::marker::PhantomData<$generic_ty>>::new(::core::ptr::null_mut())
                    };
                    static __END: ::core::sync::atomic::AtomicPtr::<::core::marker::PhantomData<$generic_ty>> = unsafe {
                        ::core::sync::atomic::AtomicPtr::<::core::marker::PhantomData<$generic_ty>>::new(::core::ptr::null_mut())
                    };

                    $crate::__support::ident_concat!((#[no_mangle]pub extern "C" fn) (register_link_section_ $ident) ((data_ptr: *const u8, data_len: usize) {
                        unsafe {
                            __START.store(data_ptr as *mut ::core::marker::PhantomData<$generic_ty>, ::core::sync::atomic::Ordering::Relaxed);
                            __END.store(data_ptr.add(data_len) as *mut ::core::marker::PhantomData<$generic_ty>, ::core::sync::atomic::Ordering::Relaxed);
                        }
                    }));

                    (&__START, &__END)
                }
            }
        }

        /// On WASM, we use an atomic pointer to the start and end of the
        /// section. The host environment is responsible for registering the
        /// section with the runtime.
        pub type SectionPtr<T> =
            &'static ::core::sync::atomic::AtomicPtr<::core::marker::PhantomData<T>>;
    }

    #[cfg(all(not(miri), target_vendor = "pc"))]
    mod section {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __get_section {
            (name=$ident:ident, type=$generic_ty:ty, aux=$($aux:ident)?) => {
                {
                    $crate::__support::add_section_link_attribute!(
                        data start $ident $($aux)?
                        #[link_section = __]
                        static __START: [$generic_ty; 0] = [];
                    );
                    $crate::__support::add_section_link_attribute!(
                        data end $ident $($aux)?
                        #[link_section = __]
                        static __END: [$generic_ty; 0] = [];
                    );

                    (
                        unsafe { &raw const __START as $crate::__support::SectionPtr<$generic_ty> },
                        unsafe { &raw const __END as $crate::__support::SectionPtr<$generic_ty> },
                    )
                }
            }
        }

        /// On Windows platforms we don't have start/end symbols, but we do have
        /// section sorting so we drop a [T; 0] at the start and end of the
        /// section.
        pub type SectionPtr<T> = *const [T; 0];
    }

    #[cfg(all(not(miri), target_os = "aix"))]
    mod section {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __get_section {
            (name=$ident:ident, type=$generic_ty:ty, aux=$($aux:ident)?) => {
                {
                    $crate::__support::add_section_link_attribute!(
                        data start $ident $($aux)?
                        #[export_name = __]
                        $crate::__support::add_section_link_attribute!(
                            data start $ident $($aux)?
                            #[link_section = __]
                            static __START: ::core::mem::MaybeUninit<[$generic_ty; 1]> = ::core::mem::MaybeUninit::uninit();
                        );
                    );
                    $crate::__support::add_section_link_attribute!(
                        data end $ident $($aux)?
                        #[export_name = __]
                        $crate::__support::add_section_link_attribute!(
                            data end $ident $($aux)?
                            #[link_section = __]
                            static __END:  ::core::mem::MaybeUninit<[$generic_ty; 1]> = ::core::mem::MaybeUninit::uninit();
                        );
                    );

                    (
                        unsafe { &raw const __START as $crate::__support::SectionPtr<$generic_ty> },
                        unsafe { &raw const __END as $crate::__support::SectionPtr<$generic_ty> },
                    )
                }
            }
        }

        /// On Windows platforms we don't have start/end symbols, but we do have
        /// section sorting so we drop a [T; 0] at the start and end of the
        /// section.
        pub type SectionPtr<T> = *const [T; 0];
    }

    #[cfg(all(not(miri), not(target_family = "wasm"), not(target_vendor = "pc"), not(target_os = "aix")))]
    mod section {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! __get_section {
            (name=$ident:ident, type=$generic_ty:ty, aux=$($aux:ident)?) => {
                {
                    extern "C" {
                        $crate::__support::add_section_link_attribute!(
                            data start $ident $($aux)?
                            #[link_name = __]
                            static __START: $crate::__support::SectionPtr<$generic_ty>;
                        );
                    }
                    extern "C" {
                        $crate::__support::add_section_link_attribute!(
                            data end $ident $($aux)?
                            #[link_name = __]
                            static __END: $crate::__support::SectionPtr<$generic_ty>;
                        );
                    }

                    (
                        unsafe { &raw const __START as $crate::__support::SectionPtr<$generic_ty> },
                        unsafe { &raw const __END as $crate::__support::SectionPtr<$generic_ty> },
                    )
                }
            }
        }

        /// On LLVM/GCC platforms we can use orphan sections with _start and
        /// _end symbols.
        pub type SectionPtr<T> = *const ::core::marker::PhantomData<T>;
    }

    /// Export a symbol into a link section.
    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_parse {
        // Needs to handle:
        //  <name>
        //  :: <name>
        //  <path> :: <name>
        //  :: <path> :: <name>
        //  etc...
        (#[in_section(unsafe, type = $stored_ty:ty, name = $ident:ident $( , aux = $aux:ident)?)] $($item:tt)*) => {
            $crate::__support::in_section_crate!((type = $stored_ty), $ident, $($aux)?, $ident, ($($item)*));
        };
        (#[in_section(unsafe, name = $ident:ident $( , aux = $aux:ident)?)] $($item:tt)*) => {
            $crate::__support::in_section_crate!(data, $ident, $($aux)?, $ident, ($($item)*));
        };
        (#[in_section( $($path:tt)* )] $($item:tt)*) => {
            $crate::__support::in_section_parse!(path=[$($path)*] #[in_section($($path)*)] $($item)*);
        };
        (path=[$orig_path:path] #[in_section($name:ident)] $($item:tt)*) => {
            $orig_path ! (
                (v=0 (name=$name (path=[$orig_path] (item=($($item)*) ()))))
            );
        };
        (path=[$orig_path:path] #[in_section(:: $($path:ident)::*)] $($item:tt)*) => {
            $crate::__support::in_section_parse!(path=[$orig_path] #[in_section($($path)::*)] $($item)*);
        };
        (path=[$orig_path:path] #[in_section($prefix:ident :: $($path:ident)::*)] $($item:tt)*) => {
            $crate::__support::in_section_parse!(path=[$orig_path] #[in_section($($path)::*)] $($item)*);
        };
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_helper_macro_generic {
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt))))) => {
            $crate::__support::in_section_crate!(section, $ident,, $path, $item);
        };
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt)))) ((aux=$aux:ident)) )=> {
            $crate::__support::in_section_crate!(section, $ident, $aux, $path, $item);
        };
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt)))) () )=> {
            $crate::__support::in_section_crate!(section, $ident,, $path, $item);
        };
        (v=$v:literal $rest:tt) => {
            const _: () = {
                compile_error!(concat!(
                    "link-section: Unsupported version: `",
                    stringify!($v),
                    "`: ",
                    stringify!($rest)
                ));
            };
        };
    }

    #[macro_export]
    #[doc(hidden)]
    macro_rules! __in_section_helper_macro_no_generic {
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt))))) => {
            $crate::__support::in_section_crate!(data, $ident,, $path, $item);
        };
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt)))) ((aux=$aux:ident)) )=> {
            $crate::__support::in_section_crate!(data, $ident, $aux, $path, $item);
        };
        ((v=0 (name=$ident:ident (path=[$path:path] (item=$item:tt $rest:tt)))) () )=> {
            $crate::__support::in_section_crate!(data, $ident,, $path, $item);
        };
        (v=$v:literal $rest:tt) => {
            const _: () = {
                compile_error!(concat!(
                    "link-section: Unsupported version: `",
                    stringify!($v),
                    "`: ",
                    stringify!($rest)
                ));
            };
        };
    }

    #[macro_export]
    #[doc(hidden)]
    #[allow(unknown_lints, edition_2024_expr_fragment_specifier)]
    macro_rules! __in_section_crate {
        (@type_select section $path:path, $item_ty:ty) => {
            <$path as $crate::__support::SectionItemType>::Item
        };
        (@type_select data $path:path, $item_ty:ty) => {
            $item_ty
        };
        (@type_select (type = $stored_ty:ty) $path:path, $item_ty:ty) => {
            $stored_ty
        };
        ($type_source:tt, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis fn $ident_fn:ident($($args:tt)*) $(-> $ret:ty)? $body:block)) => {
            $crate::__in_section_crate!(
                @inner $ident $($aux)?,
                $(#[$meta])*
                #[used]
                #[allow(non_upper_case_globals)]
                $vis static $ident_fn: $crate::__in_section_crate!(@type_select $type_source $path, fn($($args)*) $(-> $ret)?) =
                    {
                        $crate::__add_section_link_attribute!(
                            code section $ident $($aux)?
                            #[link_section = __]
                            fn $ident_fn($($args)*) $(-> $ret)? $body
                        );
                        $ident_fn
                    };
            );
        };
        ($type_source:tt, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis static _ : $ty:ty = $value:expr;)) => {
            const _: () = {
                $crate::__in_section_crate!(
                    @inner $ident $($aux)?,
                    $(#[$meta])* #[used] $vis static ANONYMOUS: $crate::__in_section_crate!(@type_select $type_source $path, $ty) = $value;
                );
            };
        };
        ($type_source:tt, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $vis:vis static $ident_static:ident : $ty:ty = $value:expr;)) => {
            $crate::__in_section_crate!(
                @inner $ident $($aux)?,
                $(#[$meta])* #[used] $vis static $ident_static: $crate::__in_section_crate!(@type_select $type_source $path, $ty) = $value;
            );
        };
        (data, $ident:ident, $($aux:ident)?, $path:path, ($(#[$meta:meta])* $item:item)) => {
            $crate::__in_section_crate!(
                @inner $ident $($aux)?, 
                $(#[$meta])* #[used] $item
            );
        };
        (@inner $ident:ident $($aux:ident)?, $item:item) => {
            $crate::__add_section_link_attribute!(
                data section $ident $($aux)?
                #[export_name = __]
                $crate::__add_section_link_attribute!(
                    data section $ident $($aux)?
                    #[link_section = __]
                    $item
                );
            );
        }
    }

    pub trait SectionItemType {
        type Item;
    }

    impl<T> SectionItemType for super::TypedSection<T> {
        type Item = T;
    }

    pub use section::SectionPtr;
}

/// Define a link section.
///
/// The definition site generates two items: a static section struct that is
/// used to access the section, and a macro that is used to place items into the
/// section. The macro is used by the [`in_section`] procedural macro.
///
/// # Attributes
///
/// - `no_macro`: Does not generate the submission macro at the definition site.
///   This will require any associated [`in_section`] invocations to use the raw
///   name of the section.
/// - `aux = <name>`: Specifies that this section is an auxiliary section, and
///   that the section is named `<name>+<aux>`.
///
/// # Example
/// ```rust
/// use link_section::{in_section, section};
///
/// #[section]
/// pub static DATA_SECTION: link_section::Section;
///
/// #[in_section(DATA_SECTION)]
/// pub fn data_function() {
///     println!("data_function");
/// }
/// ```
#[cfg(feature = "proc_macro")]
pub use ::link_section_proc_macro::section;

/// Place an item into a link section.
///
/// # Functions and typed sections
///
/// As a special case, since function declarations by themselves are not sized,
/// functions in typed sections are split and stored as function pointers.
#[cfg(feature = "proc_macro")]
pub use ::link_section_proc_macro::in_section;

/// An untyped link section that can be used to store any type. The underlying
/// data is not enumerable.
#[repr(C)]
pub struct Section {
    name: &'static str,
    start: __support::SectionPtr<()>,
    end: __support::SectionPtr<()>,
}

impl Section {
    #[doc(hidden)]
    pub const unsafe fn new(
        name: &'static str,
        start: __support::SectionPtr<()>,
        end: __support::SectionPtr<()>,
    ) -> Self {
        Self { name, start, end }
    }

    /// The byte length of the section.
    pub fn byte_len(&self) -> usize {
        unsafe { (self.end_ptr() as *const u8).offset_from(self.start_ptr() as *const u8) as usize }
    }
}

#[cfg(target_family = "wasm")]
impl Section {
    /// The start address of the section.
    pub fn start_ptr(&self) -> *const () {
        let ptr = self.start.load(::core::sync::atomic::Ordering::Relaxed) as *const ();
        if ptr.is_null() {
            panic!(
                "Section {} was not initialized by the host environment",
                self.name
            );
        }
        ptr
    }
    /// The end address of the section.
    pub fn end_ptr(&self) -> *const () {
        let ptr = self.end.load(::core::sync::atomic::Ordering::Relaxed) as *const ();
        if ptr.is_null() {
            panic!(
                "Section {} was not initialized by the host environment",
                self.name
            );
        }
        ptr
    }
}

#[cfg(not(target_family = "wasm"))]
impl Section {
    /// The start address of the section.
    pub fn start_ptr(&self) -> *const () {
        self.start as *const ()
    }
    /// The end address of the section.
    pub fn end_ptr(&self) -> *const () {
        self.end as *const ()
    }
}

impl ::core::fmt::Debug for Section {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Section")
            .field("name", &self.name)
            .field("start", &self.start_ptr())
            .field("end", &self.end_ptr())
            .field("byte_len", &self.byte_len())
            .finish()
    }
}

unsafe impl Sync for Section {}
unsafe impl Send for Section {}

/// A typed link section that can be used to store any sized type. The
/// underlying data is enumerable.
#[repr(C)]
pub struct TypedSection<T: 'static> {
    name: &'static str,
    start: __support::SectionPtr<T>,
    end: __support::SectionPtr<T>,
    _phantom: ::core::marker::PhantomData<T>,
}

#[cfg(target_family = "wasm")]
impl<T: 'static> TypedSection<T> {
    /// The start address of the section.
    pub fn start_ptr(&self) -> *const T {
        let ptr = self.start.load(::core::sync::atomic::Ordering::Relaxed) as *const T;
        if ptr.is_null() {
            panic!(
                "TypedSection {} was not initialized by the host environment",
                self.name
            );
        }
        ptr
    }

    /// The end address of the section.
    pub fn end_ptr(&self) -> *const T {
        let ptr = self.end.load(::core::sync::atomic::Ordering::Relaxed) as *const T;
        if ptr.is_null() {
            panic!(
                "TypedSection {} was not initialized by the host environment",
                self.name
            );
        }
        ptr
    }
}

#[cfg(not(target_family = "wasm"))]
impl<T: 'static> TypedSection<T> {
    /// The start address of the section.
    #[inline(always)]
    pub fn start_ptr(&self) -> *const T {
        self.start as usize as *const T
    }

    /// The end address of the section.
    #[inline(always)]
    pub fn end_ptr(&self) -> *const T {
        self.end as *const T
    }
}

// Non-const, shared functions (or functions that don't depend on the pointers)
impl<T: 'static> TypedSection<T> {
    #[doc(hidden)]
    pub const unsafe fn new(
        name: &'static str,
        start: __support::SectionPtr<T>,
        end: __support::SectionPtr<T>,
    ) -> Self {
        Self {
            name,
            start,
            end,
            _phantom: ::core::marker::PhantomData,
        }
    }

    /// The stride of the typed section.
    pub const fn stride(&self) -> usize {
        assert!(
            ::core::mem::size_of::<T>() > 0
                && ::core::mem::size_of::<T>() * 2 == ::core::mem::size_of::<[T; 2]>()
        );
        ::core::mem::size_of::<T>()
    }

    /// The byte length of the section.
    pub fn byte_len(&self) -> usize {
        unsafe { (self.end_ptr() as *const u8).offset_from(self.start_ptr() as *const u8) as usize }
    }

    /// The number of elements in the section.
    pub fn len(&self) -> usize {
        self.byte_len() / self.stride()
    }

    /// True if the section is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The section as a slice.
    pub fn as_slice(&self) -> &[T] {
        if self.is_empty() {
            &[]
        } else {
            unsafe { ::core::slice::from_raw_parts(self.start_ptr(), self.len()) }
        }
    }

    /// The offset of the item in the section, if it is in the section.
    pub fn offset_of(&self, item: &T) -> Option<usize> {
        let ptr = item as *const T;
        if ptr < self.start_ptr() || ptr >= self.end_ptr() {
            None
        } else {
            Some(unsafe { ptr.offset_from(self.start_ptr()) as usize })
        }
    }

    /// The section as a mutable slice.
    ///
    /// # Safety
    ///
    /// This cannot be safely used and is _absolutely unsound_ if any other
    /// slices are live.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn as_mut_slice(&self) -> &mut [T] {
        if self.is_empty() {
            &mut []
        } else {
            unsafe { ::core::slice::from_raw_parts_mut(self.start_ptr() as *mut T, self.len()) }
        }
    }

    /// The start address of the section.
    #[inline(always)]
    pub fn start_ptr_mut(&self) -> *mut T {
        self.start_ptr() as *mut T
    }

    /// The start address of the section.
    #[inline(always)]
    pub fn end_ptr_mut(&self) -> *mut T {
        self.end_ptr() as *mut T
    }
}

impl<'a, T> ::core::iter::IntoIterator for &'a TypedSection<T> {
    type Item = &'a T;
    type IntoIter = ::core::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<T> ::core::ops::Deref for TypedSection<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> ::core::fmt::Debug for TypedSection<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("TypedSection")
            .field("name", &self.name)
            .field("start", &self.start_ptr())
            .field("end", &self.end_ptr())
            .field("len", &self.len())
            .field("stride", &self.stride())
            .finish()
    }
}

unsafe impl<T> Sync for TypedSection<T> where T: Sync {}
unsafe impl<T> Send for TypedSection<T> where T: Send {}
