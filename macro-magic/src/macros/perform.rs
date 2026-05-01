//! A set of macros to perform chained macro invocations.

/// The top-level macro that invokes one sub-macro and renders that output.
#[macro_export]
#[doc(hidden)]
macro_rules! __perform {
    ( $input:tt, $macro:path $( [ $($args:tt)* ] )? ) => {
        $macro ! ( @entry next=$crate::__perform[[@complete]], input=$input $(, args=[$($args)*])? );
    };
    ( [@complete], ($($input:tt)*) ) => {
        $($input)*
    };
    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Pass-through macro that stringifies the input for debugging.
#[macro_export]
#[doc(hidden)]
macro_rules! __debug {
    ( @entry next=$next:path[$next_args:tt], input=$input:tt ) => {
        const _: () = { stringify! $input ; };
        $next ! ( $next_args, $input );
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Stringifies the input.
#[macro_export]
#[doc(hidden)]
macro_rules! __stringify {
    ( @entry next=$next:path[$next_args:tt], input=$input:tt, args=[$(prefix=[$($prefix:tt)*])? $(suffix=[$($suffix:tt)*])?] ) => {
        $next ! ( $next_args, ($($($prefix)*)? stringify! $input $($($suffix)*)? ) );
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Emits the arguments, ignoring the inputs.
#[macro_export]
#[doc(hidden)]
macro_rules! __emit {
    ( @entry next=$next:path[$next_args:tt], input=$input:tt, args=[$($args:tt)*] ) => {
        $next ! ( $next_args, ($($args)*) );
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Surrounds the input with the arguments.
#[macro_export]
#[doc(hidden)]
macro_rules! __surround {
    ( @entry next=$next:path[$next_args:tt], input=($($input:tt)*),
        args=[ $( prefix=[$($prefix:tt)*] )? $( suffix=[$($suffix:tt)*] )? ] ) => {
        $next ! ( $next_args, ( $($($prefix)*)? $($input)* $($($suffix)*)? ) );
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Surrounds the input with the arguments.
#[macro_export]
#[doc(hidden)]
macro_rules! __brace {
    ( @entry next=$next:path[$next_args:tt], input=($($input:tt)*), args=[ () ] ) => {
        $next ! ( $next_args, ( ( $($input)* ) ) );
    };

    ( @entry next=$next:path[$next_args:tt], input=($($input:tt)*), args=[ [] ] ) => {
        $next ! ( $next_args, ( [ $($input)* ] ) );
    };

    ( @entry next=$next:path[$next_args:tt], input=($($input:tt)*), args=[ {} ] ) => {
        $next ! ( $next_args, ( { $($input)* } ) );
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Removes surrounding braces from the input.
#[macro_export]
#[doc(hidden)]
macro_rules! __unbrace {
    ( @entry next=$next:path[$next_args:tt], input=(($($input:tt)*))) => {
        $next ! ( $next_args, ( $($input)* ) );
    };

    ( @entry next=$next:path[$next_args:tt], input=([$($input:tt)*])) => {
        $next ! ( $next_args, [ $($input)* ] );
    };

    ( @entry next=$next:path[$next_args:tt], input=({$($input:tt)*})) => {
        $next ! ( $next_args, { $($input)* } );
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Passes the input through unchanged.
#[macro_export]
#[doc(hidden)]
macro_rules! __identity {
    ( @entry next=$next:path[$next_args:tt], input=$input:tt ) => {
        $next!($next_args, $input);
    };
}

/// Sinks the input, ignoring any chained output
#[macro_export]
#[doc(hidden)]
macro_rules! __sink {
    ( @entry next=$next:path[$next_args:tt], input=$input:tt ) => {};
}

/// Splits the input and processes it down multiple paths, in order. The next
/// macro will be called for each path.
#[macro_export]
#[doc(hidden)]
macro_rules! __split {
    ( @entry next=$next:path[$next_args:tt], input=$input:tt, args=[
        $($macro:path $([$($args:tt)*])?),*
        $(,)?
    ] ) => {
        $(
            $macro ! ( @entry next=$next[$next_args], input=$input $(, args=[$($args)*])? );
        )*
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Runs the same input through multiple macros and passes the output of each to
/// the next macro concatenated.
#[macro_export]
#[doc(hidden)]
macro_rules! __parallel {
    // Entry point, start with empty accumulator
    ( @entry next=$next:path[$next_args:tt], input=$input:tt, args=$args:tt ) => {
        $crate::__parallel!( @process $input, $args, (), [$next[$next_args]] );
    };

    // Exit point, all parallel is done, emit accumulator to next macro
    ( @process $input:tt, [], $accum:tt, [$next:path[$next_args:tt]] ) => {
        $next ! ( $next_args, $accum );
    };

    // Continue point, call the next macro in the parallel chain
    ( @process $input:tt, [$next:path $([$($args:tt)*])?, $($stack:tt)*], $accum:tt, $final:tt ) => {
        $next!(
            @entry next=$crate::__parallel[[@continue [$($stack)*] $input $accum $final]],
            input=$input $(, args=[$($args)*])?
        );
    };

    ( [@continue [$($stack:tt)*] $input:tt ($( $accum:tt )*) $final:tt], ($($output:tt)*) ) => {
        $crate::__parallel!( @process $input, [$($stack)*], ($($accum)* $($output)*), $final);
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Processes input through a chain of macros, emitting the final output to the
/// next macro.
#[macro_export]
#[doc(hidden)]
macro_rules! __chain {
    // Entry point
    ( @entry next=$next:path[$next_args:tt], input=$input:tt, args= [$nextc:path $([$($argsc:tt)*])?, $($stack:tt)*] ) => {
        $nextc!( @entry next=$crate::__chain[[@continue [$($stack)*] [$next[$next_args]]]], input=$input $(, args=[$($argsc)*])?);
    };

    // Continue point, call the next macro in the chain
    ( @process $input:tt, [$next:path $([$($args:tt)*])?, $($stack:tt)*], $final:tt ) => {
        $next!( @entry next=$crate::__chain[[@continue [$($stack)*] $final]], input=$input $(, args=[$($args)*])?);
    };

    // Return from macro call, complete
    ( [@continue [] [$next:path[$next_args:tt]]], $input:tt ) => {
        $next ! ( $next_args, $input );
    };
    ( [@continue [$next:path $([$($args:tt)*])?, $($stack:tt)*] $final:tt], $input:tt ) => {
        $next!( @entry next=$crate::__chain[[@continue [$($stack)*] $final]], input=$input $(, args=[$($args)*])?);
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Separates the token trees of the input into multiple paths and runs them in parallel.
#[macro_export]
#[doc(hidden)]
macro_rules! __separate {
    // Entry point, start with empty accumulator
    ( @entry next=$next:path[$next_args:tt], input=$input:tt, args=$args:tt ) => {
        $crate::__separate!( @process $input, $args, (), [$next[$next_args]] );
    };

    // Exit point, all parallel is done, emit accumulator to next macro
    ( @process $input:tt, [], $accum:tt, [$next:path[$next_args:tt]] ) => {
        $next ! ( $next_args, $accum );
    };

    // Continue point, call the next macro in the parallel chain
    ( @process ($input:tt $($input_rest:tt)*), [$next:path $([$($args:tt)*])?, $($stack:tt)*], $accum:tt, $final:tt ) => {
        $next!(
            @entry next=$crate::__separate[[@continue [$($stack)*] ($($input_rest)*) $accum $final]],
            input=$input $(, args=[$($args)*])?
        );
    };

    ( [@continue [$($stack:tt)*] $input:tt ($( $accum:tt )*) $final:tt], ($($output:tt)*) ) => {
        $crate::__separate!( @process $input, [$($stack)*], ($($accum)* $($output)*), $final);
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Zip up multiple arrays into a single array of tuples. The arrays must all be the same length.
#[macro_export]
#[doc(hidden)]
macro_rules! __zip {
    ( @entry next=$next:path[$next_args:tt], input=$input:tt ) => {
        $crate::__zip!( @process accum=(), input=$input, next=[$next[$next_args]] );
    };

    ( @process accum=$accum:tt, input=($(())*), next=[$next:path[$next_args:tt]] ) => {
        $next ! ( $next_args, $accum );
    };

    ( @process accum=($($accum:tt)*), input=(
        $( ($first:tt $($inner:tt)*) )*
    ), next=$next:tt ) => {
        $crate::__zip!( @process accum=($($accum)* ($($first)*)), input=($(($($inner)*))*), next=$next );
    };

    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($($input)*))); };
    };
}

/// Re-arrange items from the input into a new order.
#[macro_export]
#[doc(hidden)]
macro_rules! __pick {
    // Lower-recursion path for single items
    ( @entry next=$next:path[$next_args:tt], input=($i0:tt $($rest:tt)*), args=[0]) => {
        $next ! ( $next_args, ($i0) );
    };
    ( @entry next=$next:path[$next_args:tt], input=($i0:tt $i1:tt $($rest:tt)*), args=[1]) => {
        $next ! ( $next_args, ($i1) );
    };
    ( @entry next=$next:path[$next_args:tt], input=($i0:tt $i1:tt $i2:tt $($rest:tt)*), args=[2]) => {
        $next ! ( $next_args, ($i2) );
    };

    ( @entry next=$next:path[$next_args:tt], input=$input:tt, args=$args:tt ) => {
        $crate::__pick!( @process $input, $input, (), $args, [$next[$next_args]] );
    };

    ( @process $input:tt, $input_:tt, $accum:tt, [], [$next:path[$next_args:tt]] ) => {
        $next ! ( $next_args, $accum );
    };

    ( @process ($i0:tt $($rest:tt)*), $input:tt, ($($accum:tt)*), [0 $($args:tt)*], $next:tt) => {
        $crate::__pick!( @process $input, $input, ($($accum)* $i0), [$($args)*], $next );
    };
    ( @process ($i0:tt $i1:tt $($rest:tt)*), $input:tt, ($($accum:tt)*), [1 $($args:tt)*], $next:tt) => {
        $crate::__pick!( @process $input, $input, ($($accum)* $i1), [$($args)*], $next );
    };
    ( @process ($i0:tt $i1:tt $i2:tt $($rest:tt)*), $input:tt, ($($accum:tt)*), [2 $($args:tt)*], $next:tt) => {
        $crate::__pick!( @process $input, $input, ($($accum)* $i2), [$($args)*], $next );
    };
    ( @process ($i0:tt $i1:tt $i2:tt $i3:tt $($rest:tt)*), $input:tt, ($($accum:tt)*), [3 $($args:tt)*], $next:tt) => {
        $crate::__pick!( @process $input, $input, ($($accum)* $i3), [$($args)*], $next );
    };
    ( @process ($i0:tt $i1:tt $i2:tt $i3:tt $i4:tt $($rest:tt)*), $input:tt, ($($accum:tt)*), [4 $($args:tt)*], $next:tt) => {
        $crate::__pick!( @process $input, $input, ($($accum)* $i4), [$($args)*], $next );
    };
    ( @process ($i0:tt $i1:tt $i2:tt $i3:tt $i4:tt $i5:tt $($rest:tt)*), $input:tt, ($($accum:tt)*), [5 $($args:tt)*], $next:tt) => {
        $crate::__pick!( @process $input, $input, ($($accum)* $i5), [$($args)*], $next );
    };
}

/// Calls a macro for each tokentree in the input and accumulates the results.
#[macro_export]
#[doc(hidden)]
macro_rules! __for_each {
    // Entry, empty
    ( @entry next=$next:path[$next_args:tt], input=(), args=[$macro_name:path $([$($args:tt)*])?] ) => {
        $next ! ( $next_args, () );
    };
    // Entry, dispatch first item
    ( @entry next=$next:path[$next_args:tt], input=($input:tt $($rest:tt)*), args=[$macro_name:path $([$($macro_args:tt)*])?] ) => {
        $macro_name ! ( @entry next=$crate::__for_each[
            [@continue (), ($($rest)*), [$macro_name $( [$($macro_args)*] )?], [$next[$next_args]]]
        ], input=($input) $(, args=[$($macro_args)*])? );
    };
    // Continue, still more
    ( [@continue ($($accum:tt)*), ($input:tt $($rest:tt)*), [$macro_name:path $( [$($macro_args:tt)*] )?], $next:tt], ($($output:tt)*) ) => {
        $macro_name ! ( @entry next=$crate::__for_each[
            [@continue ($($accum)* $($output)*), ($($rest)*), [$macro_name $( [$($macro_args)*] )?], $next]
        ], input=($input) $(, args=[$($macro_args)*])? );
    };
    // Continue, done
    ( [@continue ($($accum:tt)*), (), $macro:tt, [$next:path[$next_args:tt]]], ($($output:tt)*) ) => {
        $next ! ( $next_args, ($($accum)* $($output)*) );
    };
    ( $($input:tt)* ) => {
        const _: () = { compile_error!(concat!("Unexpected input for __for_each: ", stringify!($($input)*))); };
    };
}

/// Calls a dynamic macro with the given input and arguments.
#[macro_export]
#[doc(hidden)]
macro_rules! __call {
    ( @entry next=$next:path[$next_args:tt], input=($($input:tt)*), args=[$macro_name:path $([$($args:tt)*])?] ) => {
        $macro_name ! ( @entry next=$next[$next_args], input=($($input)*) $(, args=[$($args)*])? );
    };
}
