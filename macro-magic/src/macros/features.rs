//! A set of macros to declare features for another macro.

/// A macro that generates the appropriate feature extraction macros.
#[macro_export]
#[doc(hidden)]
macro_rules! __declare_features {
    ( $($input:tt)* ) => {
        $crate::__perform!(
            ($($input)*),
            $crate::__chain[
                $crate::__parse_feature_input[$],
                $crate::__parallel[
                    // (params)
                    $crate::__chain[
                        $crate::__pick[0],
                    ],
                    // (features)
                    $crate::__chain[
                        $crate::__pick[1],
                        $crate::__unbrace,
                        $crate::__for_each[$crate::__chain[
                            $crate::__fix_docs,
                            $crate::__fix_example_validate,
                            $crate::__process_defaults,
                            $crate::__evaluate_defaults,
                        ]],
                        $crate::__brace[()],
                    ],
                    // (features)
                    $crate::__chain[
                        $crate::__pick[1],
                        $crate::__feature_square,
                    ],
                ],
                // (params) (features) (feature_square)
                $crate::__parallel[
                    $crate::__identity,
                    $crate::__pick_doc_vars,
                ],
                $crate::__make_macros[$],
            ]
        );
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __generate_docs {
    ( $macro_parse:path ) => {
        $crate::__perform!(
            (),
            $crate::__chain[
                $macro_parse[$macro_parse => @raw],
                $crate::__make_docs,
            ]
        );
    };
}

/// Parses a single, generic item decorated with an attribute macro into the
/// main attribute features, other meta items, and the original item.
#[macro_export]
#[doc(hidden)]
macro_rules! __parse_item {
    ( @entry next=$next:path[$next_args:tt], input=($($item:tt)*), args=[$macro_name:path]) => {
        $crate::__chain!(@entry next=$next[$next_args], input=($($item)*), args=[
            $crate::__parallel[
                // Split meta from item and process them separately
                $crate::__chain[
                    $crate::__split_meta,
                    // (meta) (item)
                    $crate::__separate[
                        // (meta)
                        $crate::__parallel[
                            // Extract meta items
                            $crate::__chain[
                                // (meta)
                                $crate::__call[$macro_name[$macro_name => @self]],
                                // (self)(other)
                                $crate::__parallel[
                                    // (self)(other)
                                    $crate::__pick[0],
                                    // (self)(other)
                                    $crate::__separate[
                                        // (self)
                                        $crate::__extract_meta[$macro_name],
                                        // (other)
                                        $crate::__brace[()],
                                    ],
                                ],
                            ],
                        ],
                        // (item)
                        $crate::__identity,
                    ],
                ],
            ],
            // Assembles the final parsed item
            // input: features #[other_meta] item
            $crate::__finish_item,
        ]);
    };
}

/// Finishes the item parsing by collecting the features, meta, and item.
#[macro_export]
#[doc(hidden)]
macro_rules! __finish_item {
    ( @entry next=$next:path[$next_args:tt], input=(
        ($self:tt)
        $($feature:ident = $feature_value:tt $feature_value_what:ident,)*
        ($(#[$other_meta:meta])*)
        $($item:tt)*
    ) ) => {
        $next ! ( $next_args, (
            features = ($($feature = $feature_value: $feature_value_what,)*),
            self = $self,
            meta = ($(#[$other_meta])*),
            item = ($($item)*)
        ) );
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Splits the item input into meta and item.
#[macro_export]
#[doc(hidden)]
macro_rules! __split_meta {
    ( @entry next=$next:path[$next_args:tt], input=($($item:tt)*) ) => {
        $crate::__split_meta!(@loop meta=(), rest=($($item)*), item_check=($($item)*), next=[$next[$next_args]]);
    };

    ( @loop meta=($($metas:tt)*), rest=(#$meta:tt $($item:tt)*), item_check=($item_check:item), next=$next:tt ) => {
        $crate::__split_meta!(@loop meta=($($metas)* #$meta), rest=($($item)*), item_check=($item_check), next=$next);
    };

    ( @loop meta=($($meta:tt)*), rest=($($item:tt)*), item_check=($item_check:item), next=[$next:path[$next_args:tt]]) => {
        $next ! ( $next_args, (($($meta)*) ($($item)*)) );
    };

    ( @loop meta=($($metas:tt)*), rest=$rest:tt, item_check=not_an_item:tt, next=$next:tt ) => {
        const _: () = { compile_error!(concat!("Expected an item, got: ", stringify!($($input)*))); };
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Extracts the unsafe flag from an item extracted via `__parse_item`.
#[macro_export]
#[doc(hidden)]
macro_rules! __extract_unsafe {
    ( @entry next=$next:path[$next_args:tt], input=(
        features = $features:tt,
        self = $self:tt,
        meta = $meta:tt,
        item = ($vis:vis unsafe $($rest:tt)*)
    ) ) => {
        $next ! ( $next_args, (
            features = $features,
            self = $self,
            meta = $meta,
            unsafe = (unsafe),
            item = ($vis unsafe $($rest)*)
        ));
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        features = $features:tt,
        self = $self:tt,
        meta = $meta:tt,
        item = ($vis:vis static $ident:ident : & $lt:lifetime $ty:ty = &unsafe $($rest:tt)*)
    ) ) => {
        $next ! ( $next_args, (
            features = $features,
            self = $self,
            meta = $meta,
            unsafe = (unsafe),
            item = ($vis static $ident : & $lt $ty = &unsafe $($rest)*)
        ));
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        features = $features:tt,
        self = $self:tt,
        meta = $meta:tt,
        item = ($vis:vis static $ident:ident : & $lt:lifetime $ty:ty = unsafe $($rest:tt)*)
    ) ) => {
        $next ! ( $next_args, (
            features = $features,
            self = $self,
            meta = $meta,
            unsafe = (unsafe),
            item = ($vis static $ident : & $lt $ty = unsafe $($rest)*)
        ));
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        features = $features:tt,
        self = $self:tt,
        meta = $meta:tt,
        item = ($vis:vis static $ident:ident : $ty:ty = unsafe $($rest:tt)*)
    ) ) => {
        $next ! ( $next_args, (
            features = $features,
            self = $self,
            meta = $meta,
            unsafe = (unsafe),
            item = ($vis static $ident : $ty = unsafe $($rest)*)
        ));
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        features = $features:tt,
        self = $self:tt,
        meta = $meta:tt,
        item = $item:tt
    ) ) => {
        $next ! ( $next_args, (
            features = $features,
            self = $self,
            meta = $meta,
            unsafe = (),
            item = $item
        ));
    };
}

/// Parses a "pretty" feature specification into something easier to work with.
#[macro_export]
#[doc(hidden)]
macro_rules! __parse_feature_input {
    ( @entry next=$next:path[$next_args:tt], input=(
        $macro_name:ident: $macro_parse:ident;

        $(
            $( #[doc = $doc:literal] )*
            $feature:ident {
                $(
                    $( #[doc = r" crate"] $( #[doc = $doc_crate:literal] )* )?
                    feature: $feature_name:literal;
                )?
                $(
                    $( #[doc = r" attr"] $( #[doc = $doc_attr:literal] )* )?
                    attr: [($($attr:tt)*) => ($($attr_value:tt)*)];
                    $(
                        example: $example:literal;
                    )?
                    $(
                        validate: [$( $validate:tt ),*];
                    )?
                )?
                $( default {
                    $( ($default_expr:meta) => $default_value:tt, )*
                    _ => $default_fallback:tt $(,)?
                } )?
            };
        )*
    ), args=[$dollar:tt]) => {
        $next ! ( $next_args, (
            (
                $macro_name
                $macro_parse
            )
            (
                $(
                    (
                        feature = $feature;
                        docs = [$( $doc )*];
                        $(
                            name = $feature_name;
                            crate_docs = [$( $( $doc_crate )* )?];
                        )?
                        $(
                            attr = [($($attr)*) => ($($attr_value)*)];
                            attr_docs = [$( $( $doc_attr )* )?];
                            example = ($( ($example) )? (stringify!($($attr)*)));
                        )?
                        validate = (
                            $( $( [
                                $( $dollar $validate? )* $dollar (()) ?
                            ] )? )?
                            [$dollar $feature:tt]
                        );
                        default = [
                            $(
                                ((feature = $feature_name) => $feature)
                            )?
                            $(
                                $( (($default_expr) => $default_value) )*
                                (_ => $default_fallback)
                            )?
                            (_ => ())
                        ]
                    )
                )*
            )
        ) );
    };
}

/// Concatenate the global docs with the crate/attr docs.
#[macro_export]
#[doc(hidden)]
macro_rules! __fix_docs {
    ( @entry next=$next:path[$next_args:tt], input=(
        (
            feature = $feature:ident;
            docs = [$($docs:tt)*];
            name = $crate_name:literal;
            crate_docs = [$($crate_docs:tt)*];
            attr = $attr:tt;
            attr_docs = [$($attr_docs:tt)*];
            example = $example:tt;
            validate = $validate:tt;
            default = $default:tt
        )
    ) ) => {
        $next ! ( $next_args, ((
            feature = $feature;
            name = $crate_name;
            crate_docs = [$($docs)* $($crate_docs)*];
            attr = $attr;
            attr_docs = [$($docs)* $($attr_docs)*];
            example = $example;
            validate = $validate;
            default = $default
        )) );
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        (
            feature = $feature:ident;
            docs = [$($docs:tt)*];
            attr = $attr:tt;
            attr_docs = [$($attr_docs:tt)*];
            example = $example:tt;
            validate = $validate:tt;
            default = $default:tt
        )
    ) ) => {
        $next ! ( $next_args, ((
            feature = $feature;
            attr = $attr;
            attr_docs = [$($docs)* $($attr_docs)*];
            example = $example;
            validate = $validate;
            default = $default
        )) );
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        (
            feature = $feature:ident;
            docs = [$($docs:tt)*];
            name = $crate_name:literal;
            crate_docs = [$($crate_docs:tt)*];
            validate = $validate:tt;
            default = $default:tt
        )
    ) ) => {
        $next ! ( $next_args, ((
            feature = $feature;
            name = $crate_name;
            crate_docs = [$($docs)* $($crate_docs)*];
            validate = $validate;
            default = $default
        )) );
    };

    ( @entry next=$next:path[$next_args:tt], input=(
        (
            feature = $feature:ident;
            docs = [$($docs:tt)*];
            validate = $validate:tt;
            default = $default:tt
        )
    ) ) => {
        $next ! ( $next_args, ((
            feature = $feature;
            validate = $validate;
            default = $default
        )) );
    }
}

/// Ensures the example and validation are correct.
#[macro_export]
#[doc(hidden)]
macro_rules! __fix_example_validate {
    ( @entry next=$next:path[$next_args:tt], input=(
        (
            feature = $feature:ident;
            $(
                name = $crate_name:literal;
                crate_docs = $crate_docs:tt;
            )?
            $(
                attr = $attr:tt;
                attr_docs = $attr_docs:tt;
                example = ($example:tt $( ($($example_extra:tt)*) )?);
            )?
            validate = ($validate:tt $( [$($validate_extra:tt)*] )?);
            default = $default:tt
        )
    ) ) => {
        $next ! ( $next_args, ((
            feature = $feature;
            $(
                name = $crate_name;
                crate_docs = $crate_docs;
            )?
            $(
                attr = $attr;
                attr_docs = $attr_docs;
                example = $example;
            )?
            validate = $validate;
            default = $default
        )) );
    };
}

/// Process the defaults into full cfg chains.
#[macro_export]
#[doc(hidden)]
macro_rules! __process_defaults {
    ( @entry next=$next:path[$next_args:tt], input=(
        (
            feature = $feature:ident;
            $(
                name = $name_both:literal;
                crate_docs = $crate_docs:tt;
            )?
            $(
                attr = $attr:tt;
                attr_docs = $attr_docs:tt;
                example = $example:tt;
            )?
            validate = $validate:tt;
            default = [$($default:tt)*]
        )
    ) ) => {
        $crate::__process_defaults!( @process accum=(), negative=(), defaults=
            [
                $($default)*
            ],
            next=[$next[$next_args]],
            rest=(
                feature = $feature;
                $(
                    feature_crate = $feature;
                    name = $name_both;
                    crate_docs = $crate_docs;
                )?
                $(
                    feature_attr = $feature;
                    attr = $attr;
                    attr_docs = $attr_docs;
                    example = $example;
                )?
                validate = $validate;
                original_defaults = {$($default)*};
            )
        );
    };

    // Stop when we hit the final default.
    (@process accum=($($accum:tt)*), negative=$negative:tt, defaults=[(_ => $default_value:tt) $($ignored:tt)*], next=[$next:path[$next_args:tt]], rest=($($rest:tt)*)) => {
        $next ! ( $next_args, (($($rest)* default = [
            $($accum)*
            ((not(any $negative)) => $default_value)
        ])) );
    };

    // Accumulate the expression + negative and add to the negative.
    (@process accum=($($accum:tt)*), negative=($($negative:tt)*), defaults=[(($default_expr:meta) => $default_value:tt) $($default_rest:tt)*], $($rest:tt)*) => {
        $crate::__process_defaults!(@process
            accum=($($accum)* ((all($default_expr, not(any ($($negative)*)))) => $default_value) ),
            negative=($($negative)* $default_expr ,),
            defaults=[$($default_rest)*], $($rest)*);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __evaluate_defaults {
    ( @entry next=$next:path[$next_args:tt], input=((
        feature = $feature:ident;
        $(
            feature_crate = $feature_crate:ident;
            name = $name:literal;
            crate_docs = $crate_docs:tt;
        )?
        $(
            feature_attr = $feature_attr:ident;
            attr = $attr:tt;
            attr_docs = $attr_docs:tt;
            example = $example:tt;
        )?
        validate = $validate:tt;
        original_defaults = $original_defaults:tt;
        default = [
            $( ($default_expr:tt => $default_value:tt) )*
        ]
    ))) => {
        $crate::__evaluate_defaults!(@process next=[$next[$next_args]], input=($( ($default_expr => $default_value) )*), rest=(
            feature = $feature;
            $(
                feature_crate = $feature_crate;
                name = $name;
                crate_docs = $crate_docs;
            )?
            $(
                feature_attr = $feature_attr;
                attr = $attr;
                attr_docs = $attr_docs;
                example = $example;
            )?
            validate = $validate;
            original_defaults = $original_defaults;
        ));
    };

    (@process next=$next:tt, input=($( ($default_expr:tt => $default_value:tt) )*), rest=$rest:tt) => {
        $(
            #[cfg $default_expr]
            $crate::__evaluate_defaults!(@final $next, $rest, default = $default_value);
        )*
    };

    (@final [$next:path[$next_args:tt]], ($($rest:tt)*), default = (compile_error! $args:tt)) => {
        compile_error! $args;
    };

    (@final [$next:path[$next_args:tt]], ($($rest:tt)*), default = $default_value:tt) => {
        $next ! ( $next_args, (($($rest)* default = $default_value;)) );
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __feature_square {
    ( @entry next=$next:path[$next_args:tt], input=(
        ($((
            feature = $all:ident;
            $($ignored:tt)*
        ))*)
    ) ) => {
        $crate::__feature_square!( @loop queue=[$($all)*], mult=[$($all)*], next=$next[$next_args] );
    };
    ( @loop queue=[$($all:ident)*], mult=$mult:tt, next=$next:path[$next_args:tt] ) => {
        $next ! ( $next_args, (
            ($( ($all $mult) )*)
        ) );
    };
    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __extract_features {
    ( @entry macro=$macro_parse:path, next=$next:tt, features=$features:tt, all_features=$all_features:tt) => {
        $crate::__extract_features!(@loop accum=(), macro=$macro_parse, next=$next, features=$features, all_features=$all_features);
    };
    ( @loop accum=$accum:tt, macro=$macro_parse:path, next=($next:path[$next_args:tt]), features=(), all_features=$all_features:tt) => {
        $next ! ( $next_args, $accum );
    };
    ( @loop accum=$accum:tt, macro=$macro_parse:path, next=$next:tt, features=($feature:ident $($feature_rest:tt)*), all_features=$all_features:tt) => {
        $macro_parse!(@extract next=__extract_features[(@cont accum=$accum, macro=$macro_parse, next=$next, features=($($feature_rest)*), all_features=$all_features)] $feature $all_features);
    };
    ( (@cont accum=($($accum:tt)*), $($args:tt)*), ($name:ident = $output:tt $($extra:tt)+) ) => {
        $crate::__extract_features!((@cont accum=($($accum)*), $($args)*), ($name = $($extra)+));
    };
    ( (@cont accum=($($accum:tt)*), $($args:tt)*), ($name:ident = $output:tt) ) => {
        $crate::__extract_features!(@loop accum=($($accum)* $name = $output ,), $($args)*);
    };
    ( (@cont accum=($($accum:tt)*), $($args:tt)*), ($name:ident = ) ) => {
        $crate::__extract_features!(@loop accum=($($accum)* $name = () ,), $($args)*);
    };
}

/// Extracts the meta items from the proc-macro attribute.
///
/// This one is complex, follow along with the comments below...
#[macro_export]
#[doc(hidden)]
macro_rules! __extract_meta {
    // Start with a rule that matches all the various types of meta attributes.
    // Note that we don't fully validate here, it just needs to be able to
    // handle every type of meta attribute (ie: paths, idents, literals, token-trees).
    //
    // If we can't get a match here, the error in the next rule triggers.
    ( @entry next=$next:path[$next_args:tt], input=(
        (
            $(
                $name:ident $( ($($args:tt)*) )? $( = $value:tt $( $value_ident:ident )? $( :: $value_path:ident )* )?
            ),*
        )
    ), args=[$macro_path:path]) => {
        // This will return to us after the generated macro has finished processing.
        $macro_path!(@meta macro=$macro_path, next=$crate::__extract_meta[[@finish $macro_path, next=$next[$next_args]]] $(
            // Pass each of the attributes down, but we separate with a `, , ;`
            // sequence to help the downstream macro split things up.
            $name $( ($( $args )*) )? $( = $value $( $value_ident )? $( :: $value_path )* )? , , ;
        )*);
    };
    // If we couldn't parse one of those forms above, this path gets hit.
    ( @entry next=$next:path[$next_args:tt], input=(
        (
            $($input:tt)*
        )
    ), args=[$macro_path:path]) => {
        const _: () = {
            compile_error!(concat!("Unexpected form for meta attribute: ",
            stringify!($($input)*),
            "\n\n... expected 'attr', 'attr = value', 'attr(arg)', 'attr(arg) = value'"));
        };
    };
    // The generated macro passes this back to us with the value it parsed and
    // the default for the feature. If there was no value, we only get the
    // default. We'll emit the specified or default feature value and a flag
    // (value or default).
    //
    // If more than one value was specified for a given attribute, the rule
    // below will trigger and error out.
    //
    // If the generated macro doesn't recognize an attribute, it'll call back to
    // us with @error.
    ( [@finish $macro_path:path, next=$next:path[$next_args:tt]],
        ($(
            ( $name:ident = $value:tt $value_what:ident $( , $def_value:tt $def_value_what:ident )? )
        )*)
    ) => {
        // Pass the calculated feature back to @validate, which will continue on if all
        // features match the validate expressions.
        $macro_path!(@validate macro=$macro_path, next=$next[$next_args],
            test=( $($name = [$value],)* )
            pass=( $($name = $value $value_what,)* ) );
    };
    // Catch duplicate items.
    ( [@finish $macro_path:path, next=$next:path[$next_args:tt]],
        (
            $(( $name:ident = $value:tt $value_what:ident $( , $def_value:tt $def_value_what:ident $( $comma:tt $($rest:tt)* )? )? ))*
        )
    ) => {
        // TODO: This should show the underlying attribute rather than the internal name.
        const _: () = { compile_error!(concat!("Duplicate meta attribute: '", stringify!(
            $( $($($name = ...$comma)?)?  )*
        ))) };
    };
    // Unknown items (valid form, unrecognized pattern).
    ( @error rest=(
        $name:ident $( ($( $args:tt)*) )? $( = $value:tt )? , , ; $($rest:tt)*
    ) attrs=($($attr:tt)*)) => {
        const _: () = { compile_error!(concat!("Unexpected meta attribute: '", stringify!(
            $name $( ($( $args )*) )? $( = $value )?
        ),
        "'\n...expected one of:\n",
        stringify!($($attr)*))); };
    };
    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input in __extract_meta: ", stringify!($($input)*))); };
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __extract_self {
    ( @entry macro=$macro_parse:path, next=$next:tt, input=$input:tt) => {
        $crate::__extract_self!(@loop self=(), other=(), input=$input, macro=$macro_parse, next=$next);
    };
    ( @loop self=$self:tt, other=$other:tt, input=(), macro=$macro_parse:path, next=($next:path[$next_args:tt])) => {
        $next ! ( $next_args, ( $self $other ) );
    };
    ( @loop self=$self:tt, other=$other:tt, input=(# $meta:tt $($rest:tt)*), macro=$macro_parse:path, next=$next:tt) => {
        $macro_parse!(@self next=$crate::__extract_self[(@cont self=$self, other=$other, input=($($rest)*), macro=$macro_parse, next=$next)], input=(# $meta));
    };
    ( (@cont self=$self:tt, other=$other:tt, $($args:tt)*), (self = ($($output:tt)*)) ) => {
        $crate::__extract_self!(@loop self=($($output)*), other=$other, $($args)*);
    };
    ( (@cont self=$self:tt, other=($($other:tt)*), $($args:tt)*), (other = ($($output:tt)*)) ) => {
        $crate::__extract_self!(@loop self=$self, other=($($other)* $($output)*), $($args)*);
    };
}

/// Generates the various utility macros used to parse the actual macro under
/// the hood.
#[macro_export]
#[doc(hidden)]
macro_rules! __make_macros {
    ( @entry next=$next:path[$next_args:tt], input=(
        (
            $macro_name:ident
            $macro_parse:ident
        )
        (
            $(
                (
                    feature = $feature:ident;
                    $(
                        feature_crate = $feature_crate:ident;
                        name = $name:literal;
                        crate_docs = $crate_docs:tt;
                    )?
                    $(
                        feature_attr = $feature_attr:ident;
                        attr = [($($attr:tt)*) => ($($attr_output:tt)*)];
                        attr_docs = $attr_docs:tt;
                        example = $example:tt;
                    )?
                    validate = [$($validate:tt)*];
                    original_defaults = $original_defaults:tt;
                    default = $default_value:tt;
                )
            )*
        )
        (
            $(
                ($feature_sq_1:ident [$($feature_sq_2:ident)*])
            )*
        )
        $raw_features:tt
    ), args=[$dollar:tt]) => {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! $macro_parse {
            // @extract takes the full or partial feature tuple and extracts one
            // feature at a time. If present, the next macro receives
            // (name = $feature_value:tt), otherwise ().
            $(
                (@extract next=$next_macro:path[$next_macro_args:tt] $feature_sq_1
                    (
                        $dollar (
                        $(
                            $dollar ( $feature_sq_2 = $dollar $feature_sq_2:tt)?
                        )* ,
                        )*
                    )) => {
                    $next_macro ! ( $next_macro_args, (
                        $feature_sq_1 = $dollar ( $dollar ( $dollar $feature_sq_1 )? )*
                    ) );
                };
            )*

            (@extract next=$next_macro:path[$next_macro_args:tt] $dollar feature:ident $dollar ($dollar rest:tt)*) => {
                const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($dollar feature))); };
            };

            // @meta extracts the meta items from the proc-macro attribute. The
            // items need to be pre-processed to ensure that each one ends with
            // a comma and a semicolon to disambiguate.
            (@meta macro=$macro_path:path, next=$next_macro:path[$next_macro_args:tt]
                $dollar (
                    $($(
                        $dollar(
                            $($attr)*
                            ,
                            $dollar $feature:tt // comma
                        )?
                    )?)*
                    ;
                )*
            ) => {
                $next_macro ! ( $next_macro_args, ( $(($feature = $(
                    $dollar ( $dollar( $($attr_output)* value $dollar $feature )? )*
                )? $default_value default))* ) );
            };

            // Unrecognized, munch until end of recognized input.
            (@meta macro=$macro_path:path, next=$next_macro:path[$next_macro_args:tt]
                $dollar ($dollar rest:tt)*) => {
                $macro_path!(@metaerror macro=$macro_path, next=$next_macro[$next_macro_args] $dollar($dollar rest)*);
            };

            // Munch one item
            (@metaerror macro=$macro_path:path, next=$next_macro:path[$next_macro_args:tt]
                $($(
                    $dollar(
                        $($attr)*
                        ,
                        $dollar $feature:tt // comma
                    )?
                )?)*
                ;
                $dollar ($dollar rest:tt)*) => {
                $macro_path!(@metaerror macro=$macro_path, next=$next_macro[$next_macro_args] $dollar($dollar rest)*);
            };

            // Found the error!
            (@metaerror macro=$macro_path:path, next=$next_macro:path[$next_macro_args:tt]
                $dollar ($dollar rest:tt)*) => {
                $crate::__extract_meta!(@error rest=($dollar($dollar rest)*) attrs=($($($($attr)* ;)?)*));
            };

            // @validate ensures that all features match the validate expressions. If this doesn't
            // match, the next rule triggers.
            (@validate macro=$macro_path:path, next=$next_macro:path[$next_macro_args:tt],
                test=($(
                    $feature = [$($validate)*],
                )*)
                pass=$pass:tt
            ) => {
                $next_macro ! ( $next_macro_args, $pass );
            };

            // @validate didn't match.
            (@validate macro=$macro_path:path, next=$next_macro:path[$next_macro_args:tt],
                test=$test:tt
                pass=$pass:tt
            ) => {
                const _: () = { stringify!($test); };
                $macro_path!(@validateerror macro=$macro_path, test=$test);
            };

            // Munch one item
            (@validateerror macro=$macro_path:path, test=(
                $(
                    $dollar(
                        $feature = [$($validate)*]
                    )?
                )*
                , $dollar ($dollar rest:tt)*)
            ) => {
                // Pass, try next
                $macro_path!(@validateerror macro=$macro_path, test=($dollar($dollar rest)*));
            };

            // If we failed to munch an item, that was the bad one.
            (@validateerror macro=$macro_path:path, test=(
                $feature_name:ident = [$value_bad:tt],
                $dollar ($dollar rest:tt)*)
            ) => {
                compile_error!(concat!("Invalid attribute: ", stringify!($feature_name), " = ", stringify!($value_bad)));
            };

            // @self extracts the self-attribute from a list of attributes.
            (@self next=$next_macro:path [$next_macro_args:tt], input=(# [ $macro_name ])) => {
                $next_macro ! ( $next_macro_args, ( self = (()) ) );
            };
            (@self next=$next_macro:path [$next_macro_args:tt], input=(# [ $macro_name $args:tt ])) => {
                $next_macro ! ( $next_macro_args, ( self = ($args) ) );
            };
            (@self next=$next_macro:path [$next_macro_args:tt], input=$input:tt) => {
                $next_macro ! ( $next_macro_args, ( other = $input ) );
            };

            // Extracts all features specified in $all_features and passes them
            // to the next macro.
            (@entry next=$next_macro:path [$next_macro_args:tt],
                input=$all_features:tt,
                args=[$macro:path => @extract $features:tt]) => {
                $crate::__extract_features!(@entry macro=$macro, next=($next_macro [$next_macro_args]),
                    features=$features, all_features=$all_features);
            };

            // Extracts features from enabled crate features.
            (@entry next=$next_macro:path [$next_macro_args:tt],
                input=$all_features:tt,
                args=[$macro:path => @crate]) => {
                $next_macro ! ( $next_macro_args, (
                    $( $feature = $default_value, )*
                ) );
            };

            // Extracts the self-attribute from a list of attributes.
            (@entry next=$next_macro:path [$next_macro_args:tt],
                input=($dollar ( # $dollar attr:tt )*),
                args=[$macro:path => @self]) => {
                $crate::__extract_self!(@entry macro=$macro, next=($next_macro [$next_macro_args]), input=($dollar ( # $dollar attr )*));
            };

            // Extracts the raw features from the input and passes them to the next macro.
            (@entry next=$next_macro:path [$next_macro_args:tt],
                input=$input:tt, // ignored
                args=[$macro:path => @raw]) => {
                $next_macro ! ( $next_macro_args, $raw_features );
            };

            (@entry $dollar ($rest:tt)*) => {
                const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($dollar ($rest)*))); };
            };
        }

        $next ! ( $next_args, () );
    };
}

/// Extract a subset of the feature configuration for later documentation generation.
#[macro_export]
#[doc(hidden)]
macro_rules! __pick_doc_vars {
    ( @entry next=$next:path[$next_args:tt], input=(
        $params:tt
        (
            $(
                (
                    feature = $feature:ident;
                    $(
                        feature_crate = $feature_crate:ident;
                        name = $name:literal;
                        crate_docs = $crate_docs:tt;
                    )?
                    $(
                        feature_attr = $feature_attr:ident;
                        attr = $attr:tt;
                        attr_docs = $attr_docs:tt;
                        example = $example:tt;
                    )?
                    validate = $validate:tt;
                    original_defaults = $original_defaults:tt;
                    default = $default_value:tt;
                )
            )*
        )
        $feature_square:tt
    )) => {
        $next ! ( $next_args, (
            ($(
                (
                    feature = $feature;
                    $(
                        feature_crate = $feature_crate;
                        name = $name;
                        crate_docs = $crate_docs;
                    )?
                    $(
                        feature_attr = $feature_attr;
                        attr_docs = $attr_docs;
                        example = $example;
                    )?
                    original_defaults = $original_defaults;
                )
            )*)
        ));
    };
}

/// Generates a module named $macro_impl and inside, two crate-scoped macros:
///
/// make_crate_docs and make_attr_docs.
///
/// These macros are used to generate the documentation for the crate-scoped
/// features.
///
/// `make_crate_docs` generates docs for crate features. `make_attr_docs`
/// generates docs for proc macro attributes.
#[macro_export]
#[doc(hidden)]
macro_rules! __make_docs {
    ( @entry next=$next:path[$next_args:tt], input=(
        $(
            (
                feature = $feature:ident;
                $(
                    feature_crate = $feature_crate:ident;
                    name = $feature_name:literal;
                    crate_docs = [ $( $crate_doc_lit:literal )* ];
                )?
                $(
                    feature_attr = $feature_attr:ident;
                    attr_docs = [ $( $attr_doc_lit:literal )* ];
                    example = ($($example:tt)*);
                )?
                original_defaults = $original_defaults:tt;
            )
        )*
    )) => {
        $crate::__make_docs!(@defaults accum=(
            #![doc = "\n\n# Crate Features\n\n| Cargo feature | Description |\n| --- | --- |"]
            $(
                $(
                    #![doc = concat!("\n| `", $feature_name, "` | ", $( $crate_doc_lit, )* " |")]
                )?
            )*
            #![doc = "\n\n# Macro Attributes\n\n<table><tr><th>Attribute</th><th>Description</th></tr>\n"]
            $(
                $(
                    #![doc = concat!("\n<tr><td><code>", $( $example )*, "</code></td><td>\n\n", $( $attr_doc_lit, "\n", )*  "\n\n</td></tr>")]
                )?
            )*
            #![doc = "</table>"]
            #![doc = "\n\n# Defaults"]
        ),
            ($(
                (feature = $feature; default = $original_defaults;)
            )*)
        );
    };

    // Emits one "defaults" subsection per feature that has a non-()` default.
    (@defaults accum=($($accum:tt)*), ()) => {
        mod __generated_docs {
            $($accum)*
        }
    };

    // Hide attributes with no default.
    (@defaults accum=($($accum:tt)*), ((feature = $feature:ident; default = {(_ => ())};) $($rest:tt)*)) => {
        $crate::__make_docs!(@defaults accum=(
            $($accum)*
        ), ($($rest)*));
    };
    // Hide crate features with no default.
    (@defaults accum=($($accum:tt)*), ((feature = $feature:ident; default = {((feature = $feature_lit:literal) => $feature_default_value:ident) (_ => ())};) $($rest:tt)*)) => {
        $crate::__make_docs!(@defaults accum=(
            $($accum)*
        ), ($($rest)*));
    };

    (@defaults accum=($($accum:tt)*), ((feature = $feature:ident; default = $default_value:tt;) $($rest:tt)*)) => {
        $crate::__make_docs!(@default accum=(
            $($accum)*
            //!
            #![doc = concat!("## `", stringify!($feature), "`")]
            //!
            //! ```rust
            //! # #[cfg(false)] {
        ), ((feature = $feature; default = $default_value;) $($rest)*));
    };
    (@default accum=($($accum:tt)*), ((feature = $feature:ident; default = {
        (($($branch:tt)*) => $default_value:tt) $($branch_rest:tt)*};) $($rest:tt)*)) => {
        $crate::__make_docs!(@default accum=(
            $($accum)*
            #![doc = concat!("#[cfg(", stringify!($($branch)*), ")]")]
            //! # const _: () = { let
            #![doc = concat!(stringify!($feature), " = ", stringify!($default_value))]
            //! # ; };
            //!
        ), ((feature = $feature; default = {$($branch_rest)*};) $($rest)*));
    };
    (@default accum=($($accum:tt)*), ((feature = $feature:ident; default = {
        (_ => $default_value:tt) $($branch_rest:tt)*};) $($rest:tt)*)) => {
        $crate::__make_docs!(@defaults accum=(
            $($accum)*
            //! // default
            #![doc = concat!(stringify!($feature), " = ", stringify!($default_value))]
            //! # }
            //! ```
        ), ($($rest)*));
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}
