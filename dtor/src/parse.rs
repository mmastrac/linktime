#[macro_export]
#[doc(hidden)]
macro_rules! __dtor_parse {
    ( $($input:tt)* ) => {
        $crate::__perform!(
            ($($input)*),
            $crate::__chain[
                $crate::__parse_item[$crate::__dtor_features],
                $crate::__extract_unsafe,
                $crate::__dtor_parse_impl,
            ]
        );
    };
}

/// Parse a processed `dtor` item. This is intentionally verbose to avoid
/// excessive nesting of macro calls in user code.
#[macro_export]
#[doc(hidden)]
macro_rules! __dtor_parse_impl {
    // Step 0: Check function shape and features

    // fn
    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = $anonymous:tt : $anonymous_spec:ident,
            crate_path = $crate_path:tt : $crate_path_spec:ident,
            ctor_export_name_prefix = $ctor_export_name_prefix:tt : $ctor_export_name_prefix_spec:ident,
            ctor_link_section = $ctor_link_section:tt : $ctor_link_section_spec:ident,
            default_term_method = $default_term_method:tt : $default_term_method_spec:ident,
            default_unload_method = $default_unload_method:tt : $default_unload_method_spec:ident,
            export_name_prefix = $export_name_prefix:tt : $export_name_prefix_spec:ident,
            link_section = $link_section:tt : $link_section_spec:ident,
            method = $method:tt : $method_spec:ident,
            proc_macro = $proc_macro:tt : $proc_macro_spec:ident,
            std = $std:tt : $std_spec:ident,
            r#unsafe = $no_warn_on_missing_unsafe:tt : $no_warn_on_missing_unsafe_spec:ident,
            used_linker = $used_linker:tt : $used_linker_spec:ident,
        ),
        self = $self:tt,
        meta = $meta:tt,
        unsafe = ($($unsafe:tt)*),
        item = ($vis:vis $(unsafe)? $( extern $abi:literal )? fn $name:ident () $( -> () )? {
            $($body:tt)*
        })
    ) ) => {
        $crate::__dtor_parse_impl!(@entry next=$next[$next_args], input=(
            features = (
                anonymous = $anonymous,
                ctor_export_name_prefix = $ctor_export_name_prefix,
                ctor_link_section = $ctor_link_section,
                default_term_method = $default_term_method,
                default_unload_method = $default_unload_method,
                export_name_prefix = $export_name_prefix,
                link_section = $link_section,
                method = $method,
                no_warn_on_missing_unsafe = $no_warn_on_missing_unsafe,
                used_linker = $used_linker,
            ),
            meta = $meta,
            unsafe = ($($unsafe)*),
            item = ($vis $($unsafe)* $( extern $abi )? fn $name () {
                $($body)*
            })
        ));
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = $anonymous:tt : $anonymous_spec:ident,
            $($rest:tt)*
        ),
        self = $self:tt,
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = ($item:item)
    ) ) => {
        compile_error!("Invalid dtor function. \
            Expected a function with no args, \
            return value, or type parameters.\n\
            Valid forms are: [pub] [unsafe] [extern $abi] fn $name() { ... }");
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = $anonymous:tt : $anonymous_spec:ident,
            $($rest:tt)*
        ),
        self = $self:tt,
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = ($($item:tt)*)
    ) ) => {
        compile_error!(concat!("Invalid dtor, didn't get an item: ", stringify!($($item)*)));
    };

    // Step 1: Compute method

    // Delegate term -> default_term_method
    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = $anonymous:tt,
            ctor_export_name_prefix = $ctor_export_name_prefix:tt,
            ctor_link_section = $ctor_link_section:tt,
            default_term_method = $default_term_method:tt,
            default_unload_method = $default_unload_method:tt,
            export_name_prefix = $export_name_prefix:tt,
            link_section = $link_section:tt,
            method = term,
            no_warn_on_missing_unsafe = $no_warn_on_missing_unsafe:tt,
            used_linker = $used_linker:tt,
        ),
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = $item:tt
    ) ) => {
        // method = term
        $crate::__dtor_parse_impl(@entry next=$next[$next_args], input=(
            features = (
                anonymous = $anonymous,
                method = ($default_term_method $ctor_export_name_prefix $ctor_link_section $export_name_prefix $link_section),
                no_warn_on_missing_unsafe = $no_warn_on_missing_unsafe,
                used_linker = $used_linker,
            ),
            meta = $meta,
            unsafe = $unsafe,
            item = $item
        ));
    };

    // Delegate unload -> default_unload_method
    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = $anonymous:tt,
            ctor_export_name_prefix = $ctor_export_name_prefix:tt,
            ctor_link_section = $ctor_link_section:tt,
            default_term_method = $default_term_method:tt,
            default_unload_method = $default_unload_method:tt,
            export_name_prefix = $export_name_prefix:tt,
            link_section = $link_section:tt,
            method = unload,
            no_warn_on_missing_unsafe = $no_warn_on_missing_unsafe:tt,
            used_linker = $used_linker:tt,
        ),
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = $item:tt
    ) ) => {
        // method = unload
        $crate::__dtor_parse_impl!(@entry next=$next[$next_args], input=(
            features = (
                anonymous = $anonymous,
                method = ($default_unload_method $ctor_export_name_prefix $ctor_link_section $export_name_prefix $link_section),
                no_warn_on_missing_unsafe = $no_warn_on_missing_unsafe,
                used_linker = $used_linker,
            ),
            meta = $meta,
            unsafe = $unsafe,
            item = $item
        ));
    };

    // Other methods, pass through
    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = $anonymous:tt,
            ctor_export_name_prefix = $ctor_export_name_prefix:tt,
            ctor_link_section = $ctor_link_section:tt,
            default_term_method = $default_term_method:tt,
            default_unload_method = $default_unload_method:tt,
            export_name_prefix = $export_name_prefix:tt,
            link_section = $link_section:tt,
            method = $method:tt,
            no_warn_on_missing_unsafe = $no_warn_on_missing_unsafe:tt,
            used_linker = $used_linker:tt,
        ),
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = $item:tt
    ) ) => {
        // method = other
        $crate::__dtor_parse_impl!(@entry next=$next[$next_args], input=(
            features = (
                anonymous = $anonymous,
                method = ($method $ctor_export_name_prefix $ctor_link_section $export_name_prefix $link_section),
                no_warn_on_missing_unsafe = $no_warn_on_missing_unsafe,
                used_linker = $used_linker,
            ),
            meta = $meta,
            unsafe = $unsafe,
            item = $item
        ));
    };

    // Step 2: warn on missing unsafe

    // warn iff no_warn_on_missing_unsafe is not present AND unsafe is not present
    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = $anonymous:tt,
            method = $method:tt,
            no_warn_on_missing_unsafe = (),
            used_linker = $used_linker:tt,
        ),
        meta = $meta:tt,
        unsafe = (),
        item = $item:tt
    ) ) => {
        const _: () = {
            #[deprecated="dtor deprecation note:\n\n\
            Use of #[dtor] without `#[dtor(unsafe)]` or `unsafe fn` is deprecated. As code execution\n\
            before main is unsupported by most Rust runtime functions, these functions must be marked\n\
            `unsafe`."]
            const fn dtor_without_unsafe_is_deprecated() {}
            #[allow(unused)]
            static UNSAFE_WARNING: () = {
                dtor_without_unsafe_is_deprecated()
            };
        };

        $crate::__dtor_parse_impl!(@entry next=$next[$next_args], input=(
            features = (
                anonymous = $anonymous,
                method = $method,
                used_linker = $used_linker,
            ),
            meta = $meta,
            unsafe = (),
            item = $item
        ));
    };

    // If no_warn_on_missing_unsafe, ignore
    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = $anonymous:tt,
            method = $method:tt,
            no_warn_on_missing_unsafe = $no_warn_on_missing_unsafe:tt,
            used_linker = $used_linker:tt,
        ),
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = $item:tt
    ) ) => {
        $crate::__dtor_parse_impl!(@entry next=$next[$next_args], input=(
            features = (
                anonymous = $anonymous,
                method = $method,
                used_linker = $used_linker,
            ),
            meta = $meta,
            unsafe = $unsafe,
            item = $item
        ));
    };

    // Step 3: Wrap anonymous if needed
    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = anonymous,
            method = $method:tt,
            used_linker = $used_linker:tt,
        ),
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = $item:tt
    ) ) => {
        // Anonymous function
        const _: () = {
            $crate::__dtor_parse_impl!(@entry next=$next[$next_args], input=(
                features = (
                    method = $method,
                    used_linker = $used_linker,
                ),
                meta = $meta,
                unsafe = $unsafe,
                item = $item
            ));
        };
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            anonymous = (),
            method = $method:tt,
            used_linker = $used_linker:tt,
        ),
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = $item:tt
    ) ) => {
        $crate::__dtor_parse_impl!(@entry next=$next[$next_args], input=(
            features = (
                method = $method,
                used_linker = $used_linker,
            ),
            meta = $meta,
            unsafe = $unsafe,
            item = $item
        ));
    };

    // Step 4: Compute used_linker
    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            method = $method:tt,
            used_linker = (),
        ),
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = $item:tt
    ) ) => {
        $crate::__dtor_parse_impl!(@entry next=$next[$next_args], input=(
            features = (
                method = $method,
                used_linker_meta = (#[used]),
            ),
            meta = $meta,
            unsafe = $unsafe,
            item = $item
        ));
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            method = $method:tt,
            used_linker = used_linker,
        ),
        meta = $meta:tt,
        unsafe = $unsafe:tt,
        item = $item:tt
    ) ) => {
        $crate::__dtor_parse_impl!(@entry next=$next[$next_args], input=(
            features = (
                method = $method,
                used_linker_meta = (#[used(linker)]),
            ),
            meta = $meta,
            unsafe = $unsafe,
            item = $item
        ));
    };

    // Step 5: Delegate on method (at_module_exit, at_binary_exit, link_section)
    ( @entry next=$next:path[$next_args:tt], input=(
        features = (
            method = ($method:tt $ctor_export_name_prefix:tt $ctor_link_section:tt $export_name_prefix:tt $link_section:tt),
            used_linker_meta = (#$used_linker_meta:tt),
        ),
        meta = ($($meta:tt)*),
        unsafe = ($($unsafe:tt)*),
        item = ($vis:vis $(unsafe)? $( extern $abi:literal )? fn $name:ident $args:tt $( -> () )? {
            $($body:tt)*
        })
    ) ) => {
        $($meta)*
        #[allow(dead_code)]
        $vis $($unsafe)* $( extern $abi )? fn $name $args {
            // The outer function may be attached to a struct, so we generate an
            // inner function that is freestanding and call it from both places.
            $($unsafe)* $( extern $abi )? fn __dtor_private_inner $args {
                $($body)*
            }

            $crate::__dtor_parse_impl!(@dtor next=$next[$next_args], input=(
                features = (
                    ctor_export_name_prefix = $ctor_export_name_prefix,
                    ctor_link_section = $ctor_link_section,
                    export_name_prefix = $export_name_prefix,
                    link_section = $link_section,
                    method = $method,
                    used_linker_meta = (#$used_linker_meta),
                ),
                item = ($($unsafe)* { __dtor_private_inner() })
            ));

            $($unsafe)* { __dtor_private_inner() }
        }
    };

    // dtor

    ( @dtor next=$next:path[$next_args:tt], input=(
        features = (
            ctor_export_name_prefix = $ctor_export_name_prefix:tt,
            ctor_link_section = $ctor_link_section:tt,
            export_name_prefix = (),
            link_section = $link_section:tt,
            method = linker,
            used_linker_meta = (#$used_linker_meta:tt),
        ),
        item = ($($item:tt)*)
    ) ) => {
        const _: () = {
            #[link_section = $link_section]
            #$used_linker_meta
            static __DTOR_PRIVATE_REF: extern "C" fn() = {
                extern "C" fn __dtor_private() {
                    $($item)*
                }
                __dtor_private
            };
        };
    };

    ( @dtor next=$next:path[$next_args:tt], input=(
        features = (
            ctor_export_name_prefix = $ctor_export_name_prefix:tt,
            ctor_link_section = $ctor_link_section:tt,
            export_name_prefix = $export_name_prefix:tt,
            link_section = $link_section:tt,
            method = linker,
            used_linker_meta = (#$used_linker_meta:tt),
        ),
        item = ($($item:tt)*)
    ) ) => {
        const _: () = {
            #[no_mangle]
            #[export_name = concat!($export_name_prefix, "_",
                env!("CARGO_PKG_NAME"),
                "_",
                module_path!(),
                "_",
                stringify!($name),
                "_L",
                line!(),
                "C",
                column!())]
            extern "C" fn __dtor_private() {
                $($item)*
            }
        };
    };

    ( @dtor next=$next:path[$next_args:tt], input=(
        features = (
            ctor_export_name_prefix = (),
            ctor_link_section = $ctor_link_section:tt,
            export_name_prefix = $export_name_prefix:tt,
            link_section = $link_section:tt,
            method = $method:ident,
            used_linker_meta = (#$used_linker_meta:tt),
        ),
        item = ($($item:tt)*)
    ) ) => {
        const _: () = {
            #[link_section = $ctor_link_section]
            #$used_linker_meta
            static __CTOR_PRIVATE_REF: unsafe extern "C" fn() = {
                unsafe extern "C" fn __ctor_private() {
                    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
                    {
                        static DISARMED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);
                        if DISARMED.swap(true, ::core::sync::atomic::Ordering::Relaxed) {
                            return;
                        }
                    }
                    $crate::__support::$method(__dtor_private);
                }
                extern "C" fn __dtor_private() {
                    $($item)*
                }
                __ctor_private
            };
        };
    };

    ( @dtor next=$next:path[$next_args:tt], input=(
        features = (
            ctor_export_name_prefix = $ctor_export_name_prefix:tt,
            ctor_link_section = $ctor_link_section:tt,
            export_name_prefix = $export_name_prefix:tt,
            link_section = $link_section:tt,
            method = $method:ident,
            used_linker_meta = (#$used_linker_meta:tt),
        ),
        item = ($($item:tt)*)
    ) ) => {
        const _: () = {
            #[no_mangle]
            #[export_name = concat!($ctor_export_name_prefix, "_",
                env!("CARGO_PKG_NAME"),
                "_",
                module_path!(),
                "_",
                stringify!($name),
                "_L",
                line!(),
                "C",
                column!())]
            unsafe extern "C" fn __ctor_private() {
                $crate::__support::$method(__dtor_private);
            }

            extern "C" fn __dtor_private() {
                $($item)*
            }
        };
    };

    ( @entry next=$next:path[$next_args:tt], input=$input:tt ) => {
        compile_error!(concat!("Invalid dtor input: ", stringify!($input)));
    };
}
