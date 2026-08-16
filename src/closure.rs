/// Creates an anonymous Fn-, FnMut-, or FnOnce-like closure whose call method
/// may be generic.
///
/// # Quick guide
///
/// First declare the interface with [`closure_trait!`](crate::closure_trait),
/// then repeat that trait's receiver, generic bounds, arguments, and output in
/// the `closure!` call:
///
/// ```text
/// closure!(
///     captures,
///     Trait<T: Bounds>(&mut self, first: T, second: T) -> Output
///     where Provider: Provides<T>
///     { body }
/// )
/// ```
///
/// Captures are optional. The result is an anonymous value implementing
/// `Trait`; invoke it with `call`, `call_mut`, or `call_once` as permitted by
/// the receiver declared for that trait. The default `alloc` feature additionally
/// provides `call_box`.
///
/// Capture declarations precede the trait and method signature:
///
/// - `&x: Type` or `&mut x: Type` borrows `x` at construction;
/// - `x: Type` or `mut x: Type` moves `x` into the closure;
/// - `clone x: Type` or `clone mut x: Type` clones `x` at construction;
/// - `x: &'closure Type` or `x: &'closure mut Type` stores an existing
///   reference.
///
/// The receiver selects the call mode. An omitted receiver, or `&self`, creates
/// an Fn-like implementation. `&mut self` creates an FnMut-like implementation,
/// and `self` creates an FnOnce-like implementation. Mutable owned captures must
/// opt in with `mut`; `&mut` captures are already explicitly mutable. Call
/// arguments use `name: Type` syntax and may be omitted or repeated; a trailing
/// comma is accepted. A capture may not have the same name as a call argument.
/// Any generic `where` clause declared by `closure_trait!` is repeated before
/// the body.
///
/// Every capture requires an explicit type and a trailing comma. The
/// macro-provided `'closure` lifetime may occur anywhere in a capture type.
/// Omitting `-> Output` means `-> ()`.
///
/// ```
/// use generic_closure::{closure, closure_trait};
/// use std::fmt::Display;
///
/// closure_trait!(Render<T: Display>(value: T) -> String);
///
/// let prefix = String::from("value: ");
/// let suffix = String::from("!");
/// let render = closure!(
///     clone prefix: String,
///     suffix: String,
///     Render<T: Display>(value: T) -> String {
///         format!("{prefix}{value}{suffix}")
///     }
/// );
///
/// assert_eq!(prefix, "value: ");
/// assert_eq!(render.call(42), "value: 42!");
/// assert_eq!(render.call("hello"), "value: hello!");
/// ```
#[macro_export]
macro_rules! closure {
    ($($argument_type:tt)*) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [self];
            struct_fields = [];
            constructor_fields = [];
            ref_bindings = [];
            mut_bindings = [];
            once_bindings = [];
            mutable_captures = [];
            remaining = [$($argument_type)*];
        }
    };
}

/// Rejects call arguments that have the same name as a capture.
///
/// All names occur in one destructuring pattern, so rustc's duplicate-binding
/// check compares them without generating every argument/capture pair.
#[doc(hidden)]
#[macro_export]
macro_rules! __generic_closure_reject_shadowing {
    (
        arguments = [$($argument:ident),*];
        captures = [$($capture:ident: $capture_type:ty,)*];
    ) => {
        const _: () = {
            let (
                ($($argument,)*),
                ($($capture,)*),
            ) = (
                ($({ let _ = ::core::stringify!($argument); },)*),
                ($({ let _ = ::core::stringify!($capture); },)*),
            );
            let _ = (
                ($($argument,)*),
                ($($capture,)*),
            );
        };
    };
}

/// # Implementation guide
///
/// This token-tree muncher first accumulates capture metadata, constructor values,
/// and three parallel sets of body-binding adjustments. Captures are stored in a
/// tuple next to the lifetime marker, avoiding any reserved field name. The
/// selected receiver later destructures that tuple by shared borrow, mutable
/// borrow, or value before applying the corresponding adjustments. A separate
/// list records mutable captures so an Fn-like signature can reject them before
/// emitting Rust items.
/// Generic bounds are consumed recursively until the argument list identifies
/// their final `>`; an optional `where` clause is then consumed until the body.
/// Non-generic signatures skip those phases.
///
/// The generated local struct binds `'closure`, so capture type fragments can
/// use that lifetime at any nesting depth. The helper is exported only because a
/// public macro's expansion must be able to find it from downstream crates.
#[doc(hidden)]
#[macro_export]
macro_rules! __generic_closure_parse_closure {
    // Borrow a mutable value from the construction context.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [& mut $capture:ident: $capture_type:ty, $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [$receiver];
            struct_fields = [$($struct_fields)* $capture: &'closure mut $capture_type,];
            constructor_fields = [$($constructor_fields)* $capture: &mut $capture,];
            ref_bindings = [$($ref_bindings)* let $capture = &**$capture;];
            mut_bindings = [$($mut_bindings)* let $capture = &mut **$capture;];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)* $capture];
            remaining = [$($remaining)*];
        }
    };

    // Borrow a shared value from the construction context.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [& $capture:ident: $capture_type:ty, $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [$receiver];
            struct_fields = [$($struct_fields)* $capture: &'closure $capture_type,];
            constructor_fields = [$($constructor_fields)* $capture: &$capture,];
            ref_bindings = [$($ref_bindings)* let $capture = *$capture;];
            mut_bindings = [$($mut_bindings)* let $capture = *$capture;];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            remaining = [$($remaining)*];
        }
    };

    // Store an existing mutable reference, reborrowing it for every FnMut call.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [$capture:ident: &'closure mut $referent:ty, $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [$receiver];
            struct_fields = [$($struct_fields)* $capture: &'closure mut $referent,];
            constructor_fields = [$($constructor_fields)* $capture: $capture,];
            ref_bindings = [$($ref_bindings)* let $capture = &**$capture;];
            mut_bindings = [$($mut_bindings)* let $capture = &mut **$capture;];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)* $capture];
            remaining = [$($remaining)*];
        }
    };

    // Store an existing shared reference without exposing an extra `&` level.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [$capture:ident: &'closure $referent:ty, $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [$receiver];
            struct_fields = [$($struct_fields)* $capture: &'closure $referent,];
            constructor_fields = [$($constructor_fields)* $capture: $capture,];
            ref_bindings = [$($ref_bindings)* let $capture = *$capture;];
            mut_bindings = [$($mut_bindings)* let $capture = *$capture;];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            remaining = [$($remaining)*];
        }
    };

    // Clone a value into a mutable field.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [clone mut $capture:ident: $capture_type:ty, $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [$receiver];
            struct_fields = [$($struct_fields)* $capture: $capture_type,];
            constructor_fields = [
                $($constructor_fields)*
                $capture: ::core::clone::Clone::clone(&$capture),
            ];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)* let mut $capture = $capture;];
            mutable_captures = [$($mutable_captures)* $capture];
            remaining = [$($remaining)*];
        }
    };

    // Move a value into a mutable field.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [mut $capture:ident: $capture_type:ty, $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [$receiver];
            struct_fields = [$($struct_fields)* $capture: $capture_type,];
            constructor_fields = [$($constructor_fields)* $capture: $capture,];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)* let mut $capture = $capture;];
            mutable_captures = [$($mutable_captures)* $capture];
            remaining = [$($remaining)*];
        }
    };

    // Cloning an existing shared reference still exposes one reference level.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [clone $capture:ident: &'closure $referent:ty, $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [$receiver];
            struct_fields = [$($struct_fields)* $capture: &'closure $referent,];
            constructor_fields = [
                $($constructor_fields)*
                $capture: ::core::clone::Clone::clone(&$capture),
            ];
            ref_bindings = [$($ref_bindings)* let $capture = *$capture;];
            mut_bindings = [$($mut_bindings)* let $capture = *$capture;];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            remaining = [$($remaining)*];
        }
    };

    // Clone an immutable value once at construction.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [clone $capture:ident: $capture_type:ty, $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [$receiver];
            struct_fields = [$($struct_fields)* $capture: $capture_type,];
            constructor_fields = [
                $($constructor_fields)*
                $capture: ::core::clone::Clone::clone(&$capture),
            ];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)* let $capture = &*$capture;];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            remaining = [$($remaining)*];
        }
    };

    // Move an immutable value into the closure.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [$capture:ident: $capture_type:ty, $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_captures
            receiver = [$receiver];
            struct_fields = [$($struct_fields)* $capture: $capture_type,];
            constructor_fields = [$($constructor_fields)* $capture: $capture,];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)* let $capture = &*$capture;];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            remaining = [$($remaining)*];
        }
    };

    // A generic trait name ends the capture list.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [$trait_name:ident<$type_parameter:ident: $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_bounds
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            parsed_bounds = [];
            remaining = [$($remaining)*];
        }
    };

    // A non-generic signature is complete once its body is found.
    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [$trait_name:ident($($signature:tt)*) -> $output:ty $body:block];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_nongeneric_signature
            signature = [$($signature)*];
            output = [$output];
            body = [$body];
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
        }
    };

    (@parse_captures
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        remaining = [$trait_name:ident($($signature:tt)*) $body:block];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_nongeneric_signature
            signature = [$($signature)*];
            output = [()];
            body = [$body];
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
        }
    };

    (@parse_captures $($invalid:tt)*) => {
        ::core::compile_error!(concat!(
            "malformed closure! invocation; expected comma-terminated captures ",
            "such as `x: Type`, `mut x: Type`, `clone x: Type`, `clone mut x: ",
            "Type`, `&x: Type`, or `&mut x: Type`, followed by a call signature"
        ));
    };

    // Split the final adjacent `>>` before the call signature.
    (@parse_bounds
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>>($($signature:tt)*) $($tail:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_bounds
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            parsed_bounds = [$($parsed_bounds)+ >];
            remaining = [>($($signature)*) $($tail)*];
        }
    };

    (@parse_bounds
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>($($signature:tt)*) -> $output:ty where $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_where
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            bounds = [$($parsed_bounds)+];
            signature = [$($signature)*];
            output = [$output];
            parsed_predicates = [];
            remaining = [$($remaining)*];
        }
    };

    (@parse_bounds
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>($($signature:tt)*) where $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_where
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            bounds = [$($parsed_bounds)+];
            signature = [$($signature)*];
            output = [()];
            parsed_predicates = [];
            remaining = [$($remaining)*];
        }
    };

    (@parse_bounds
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>($($signature:tt)*) -> $output:ty $body:block];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_generic_signature
            signature = [$($signature)*];
            output = [$output];
            body = [$body];
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            bounds = [$($parsed_bounds)+];
            predicates = [];
        }
    };

    (@parse_bounds
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>($($signature:tt)*) $body:block];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_generic_signature
            signature = [$($signature)*];
            output = [()];
            body = [$body];
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            bounds = [$($parsed_bounds)+];
            predicates = [];
        }
    };

    (@parse_bounds
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)*];
        remaining = [$next:tt $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_bounds
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            parsed_bounds = [$($parsed_bounds)* $next];
            remaining = [$($remaining)*];
        }
    };

    (@parse_bounds $($invalid:tt)*) => {
        ::core::compile_error!(concat!(
            "malformed closure! generic signature; expected `Trait<T: Bounds>",
            "(first: T, second: T) -> Output`, optionally with `&self`, ",
            "`&mut self`, or `self`; omit `-> Output` for unit"
        ));
    };

    // Consume a generic method `where` clause until the closure body.
    (@parse_where
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+];
        signature = [$($signature:tt)*];
        output = [$output:ty];
        parsed_predicates = [$($predicates:tt)+];
        remaining = [$body:block];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_generic_signature
            signature = [$($signature)*];
            output = [$output];
            body = [$body];
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            bounds = [$($bounds)+];
            predicates = [$($predicates)+];
        }
    };

    (@parse_where
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+];
        signature = [$($signature:tt)*];
        output = [$output:ty];
        parsed_predicates = [$($predicates:tt)*];
        remaining = [$next:tt $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_closure! {
            @parse_where
            receiver = [$receiver];
            struct_fields = [$($struct_fields)*];
            constructor_fields = [$($constructor_fields)*];
            ref_bindings = [$($ref_bindings)*];
            mut_bindings = [$($mut_bindings)*];
            once_bindings = [$($once_bindings)*];
            mutable_captures = [$($mutable_captures)*];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            bounds = [$($bounds)+];
            signature = [$($signature)*];
            output = [$output];
            parsed_predicates = [$($predicates)* $next];
            remaining = [$($remaining)*];
        }
    };

    (@parse_where $($invalid:tt)*) => {
        ::core::compile_error!(
            "malformed closure! where clause; expected at least one predicate followed by a body"
        );
    };

    // Normalize generic receiver syntax.
    (@emit_generic_signature
        signature = [& self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty]; body = [$body:block]; $($state:tt)*
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_generic mode = [ref]; $($state)*
            arguments = [$($argument: $argument_type),*]; output = [$output]; body = [$body];
        }
    };
    (@emit_generic_signature
        signature = [& mut self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty]; body = [$body:block]; $($state:tt)*
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_generic mode = [mut]; $($state)*
            arguments = [$($argument: $argument_type),*]; output = [$output]; body = [$body];
        }
    };
    (@emit_generic_signature
        signature = [self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty]; body = [$body:block]; $($state:tt)*
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_generic mode = [once]; $($state)*
            arguments = [$($argument: $argument_type),*]; output = [$output]; body = [$body];
        }
    };
    (@emit_generic_signature
        signature = [$($argument:ident: $argument_type:ty),* $(,)?];
        output = [$output:ty]; body = [$body:block]; $($state:tt)*
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_generic mode = [ref]; $($state)*
            arguments = [$($argument: $argument_type),*]; output = [$output]; body = [$body];
        }
    };
    (@emit_generic_signature $($invalid:tt)*) => {
        ::core::compile_error!(concat!(
            "malformed closure! generic arguments; expected `name: Type` ",
            "arguments such as `(value: T)` or `(left: T, right: T)`, ",
            "optionally preceded by `&self`, `&mut self`, or `self`"
        ));
    };

    // Normalize non-generic receiver syntax.
    (@emit_nongeneric_signature
        signature = [& self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty]; body = [$body:block]; $($state:tt)*
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_nongeneric mode = [ref]; $($state)*
            arguments = [$($argument: $argument_type),*]; output = [$output]; body = [$body];
        }
    };
    (@emit_nongeneric_signature
        signature = [& mut self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty]; body = [$body:block]; $($state:tt)*
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_nongeneric mode = [mut]; $($state)*
            arguments = [$($argument: $argument_type),*]; output = [$output]; body = [$body];
        }
    };
    (@emit_nongeneric_signature
        signature = [self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty]; body = [$body:block]; $($state:tt)*
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_nongeneric mode = [once]; $($state)*
            arguments = [$($argument: $argument_type),*]; output = [$output]; body = [$body];
        }
    };
    (@emit_nongeneric_signature
        signature = [$($argument:ident: $argument_type:ty),* $(,)?];
        output = [$output:ty]; body = [$body:block]; $($state:tt)*
    ) => {
        $crate::__generic_closure_parse_closure! {
            @emit_nongeneric mode = [ref]; $($state)*
            arguments = [$($argument: $argument_type),*]; output = [$output]; body = [$body];
        }
    };
    (@emit_nongeneric_signature $($invalid:tt)*) => {
        ::core::compile_error!(concat!(
            "malformed closure! non-generic arguments; expected `name: Type` ",
            "arguments such as `(value: i32)` or `(left: i32, right: i32)`, ",
            "optionally preceded by `&self`, `&mut self`, or `self`"
        ));
    };

    // An Fn-like receiver cannot provide mutable access to captures.
    (@emit_generic mode = [ref];
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$first:ident $($rest:ident)*];
        $($tail:tt)*
    ) => {
        ::core::compile_error!(concat!(
            "mutable capture `", stringify!($first),
            "` requires a closure signature with `&mut self` or `self`"
        ));
    };
    (@emit_nongeneric mode = [ref];
        receiver = [$receiver:ident];
        struct_fields = [$($struct_fields:tt)*];
        constructor_fields = [$($constructor_fields:tt)*];
        ref_bindings = [$($ref_bindings:tt)*];
        mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*];
        mutable_captures = [$first:ident $($rest:ident)*];
        $($tail:tt)*
    ) => {
        ::core::compile_error!(concat!(
            "mutable capture `", stringify!($first),
            "` requires a closure signature with `&mut self` or `self`"
        ));
    };

    // Emit generic implementations for the three receiver modes.
    (@emit_generic mode = [ref];
        receiver = [$receiver:ident];
        struct_fields = [$($capture:ident: $capture_type:ty,)*];
        constructor_fields = [$($_constructor_capture:ident: $constructor:expr,)*];
        ref_bindings = [$($ref_bindings:tt)*]; mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*]; mutable_captures = [];
        trait_name = [$trait_name:ident]; type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+]; predicates = [$($predicates:tt)*];
        arguments = [$($argument:ident: $argument_type:ty),*];
        output = [$output:ty]; body = [$body:block];
    ) => {{
        $crate::__generic_closure_reject_shadowing! {
            arguments = [$($argument),*];
            captures = [$($capture: $capture_type,)*];
        }
        struct __GenericClosure<'closure>(
            ($($capture_type,)*),
            ::core::marker::PhantomData<&'closure ()>,
        );
        impl<'closure> $trait_name for __GenericClosure<'closure> {
            fn call<$type_parameter>(&$receiver, $($argument: $argument_type),*) -> $output
            where $type_parameter: $($bounds)+, $($predicates)*
            {
                let ($($capture,)*) = &$receiver.0;
                $($ref_bindings)*
                $body
            }
        }
        __GenericClosure(
            ($($constructor,)*),
            ::core::marker::PhantomData,
        )
    }};

    (@emit_generic mode = [mut];
        receiver = [$receiver:ident];
        struct_fields = [$($capture:ident: $capture_type:ty,)*];
        constructor_fields = [$($_constructor_capture:ident: $constructor:expr,)*];
        ref_bindings = [$($ref_bindings:tt)*]; mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*]; mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident]; type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+]; predicates = [$($predicates:tt)*];
        arguments = [$($argument:ident: $argument_type:ty),*];
        output = [$output:ty]; body = [$body:block];
    ) => {{
        $crate::__generic_closure_reject_shadowing! {
            arguments = [$($argument),*];
            captures = [$($capture: $capture_type,)*];
        }
        struct __GenericClosure<'closure>(
            ($($capture_type,)*),
            ::core::marker::PhantomData<&'closure ()>,
        );
        impl<'closure> $trait_name for __GenericClosure<'closure> {
            fn call_mut<$type_parameter>(&mut $receiver, $($argument: $argument_type),*) -> $output
            where $type_parameter: $($bounds)+, $($predicates)*
            {
                let ($($capture,)*) = &mut $receiver.0;
                $($mut_bindings)*
                $body
            }
        }
        __GenericClosure(
            ($($constructor,)*),
            ::core::marker::PhantomData,
        )
    }};

    (@emit_generic mode = [once];
        receiver = [$receiver:ident];
        struct_fields = [$($capture:ident: $capture_type:ty,)*];
        constructor_fields = [$($_constructor_capture:ident: $constructor:expr,)*];
        ref_bindings = [$($ref_bindings:tt)*]; mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*]; mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident]; type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+]; predicates = [$($predicates:tt)*];
        arguments = [$($argument:ident: $argument_type:ty),*];
        output = [$output:ty]; body = [$body:block];
    ) => {{
        $crate::__generic_closure_reject_shadowing! {
            arguments = [$($argument),*];
            captures = [$($capture: $capture_type,)*];
        }
        struct __GenericClosure<'closure>(
            ($($capture_type,)*),
            ::core::marker::PhantomData<&'closure ()>,
        );
        impl<'closure> $trait_name for __GenericClosure<'closure> {
            fn call_once<$type_parameter>($receiver, $($argument: $argument_type),*) -> $output
            where $type_parameter: $($bounds)+, $($predicates)*
            {
                let ($($capture,)*) = $receiver.0;
                $($once_bindings)*
                $body
            }
            $crate::__generic_closure_if_alloc! {
                fn call_box<$type_parameter>(
                    self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output
                where $type_parameter: $($bounds)+, $($predicates)*
                {
                    $trait_name::call_once(*self, $($argument),*)
                }
            }
        }
        __GenericClosure(
            ($($constructor,)*),
            ::core::marker::PhantomData,
        )
    }};

    // Emit non-generic implementations for the three receiver modes.
    (@emit_nongeneric mode = [ref];
        receiver = [$receiver:ident];
        struct_fields = [$($capture:ident: $capture_type:ty,)*];
        constructor_fields = [$($_constructor_capture:ident: $constructor:expr,)*];
        ref_bindings = [$($ref_bindings:tt)*]; mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*]; mutable_captures = [];
        trait_name = [$trait_name:ident]; arguments = [$($argument:ident: $argument_type:ty),*];
        output = [$output:ty]; body = [$body:block];
    ) => {{
        $crate::__generic_closure_reject_shadowing! {
            arguments = [$($argument),*];
            captures = [$($capture: $capture_type,)*];
        }
        struct __NongenericClosure<'closure>(
            ($($capture_type,)*),
            ::core::marker::PhantomData<&'closure ()>,
        );
        impl<'closure> $trait_name for __NongenericClosure<'closure> {
            fn call(&$receiver, $($argument: $argument_type),*) -> $output {
                let ($($capture,)*) = &$receiver.0;
                $($ref_bindings)*
                $body
            }
        }
        __NongenericClosure(
            ($($constructor,)*),
            ::core::marker::PhantomData,
        )
    }};

    (@emit_nongeneric mode = [mut];
        receiver = [$receiver:ident];
        struct_fields = [$($capture:ident: $capture_type:ty,)*];
        constructor_fields = [$($_constructor_capture:ident: $constructor:expr,)*];
        ref_bindings = [$($ref_bindings:tt)*]; mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*]; mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident]; arguments = [$($argument:ident: $argument_type:ty),*];
        output = [$output:ty]; body = [$body:block];
    ) => {{
        $crate::__generic_closure_reject_shadowing! {
            arguments = [$($argument),*];
            captures = [$($capture: $capture_type,)*];
        }
        struct __NongenericClosure<'closure>(
            ($($capture_type,)*),
            ::core::marker::PhantomData<&'closure ()>,
        );
        impl<'closure> $trait_name for __NongenericClosure<'closure> {
            fn call_mut(&mut $receiver, $($argument: $argument_type),*) -> $output {
                let ($($capture,)*) = &mut $receiver.0;
                $($mut_bindings)*
                $body
            }
        }
        __NongenericClosure(
            ($($constructor,)*),
            ::core::marker::PhantomData,
        )
    }};

    (@emit_nongeneric mode = [once];
        receiver = [$receiver:ident];
        struct_fields = [$($capture:ident: $capture_type:ty,)*];
        constructor_fields = [$($_constructor_capture:ident: $constructor:expr,)*];
        ref_bindings = [$($ref_bindings:tt)*]; mut_bindings = [$($mut_bindings:tt)*];
        once_bindings = [$($once_bindings:tt)*]; mutable_captures = [$($mutable_captures:ident)*];
        trait_name = [$trait_name:ident]; arguments = [$($argument:ident: $argument_type:ty),*];
        output = [$output:ty]; body = [$body:block];
    ) => {{
        $crate::__generic_closure_reject_shadowing! {
            arguments = [$($argument),*];
            captures = [$($capture: $capture_type,)*];
        }
        struct __NongenericClosure<'closure>(
            ($($capture_type,)*),
            ::core::marker::PhantomData<&'closure ()>,
        );
        impl<'closure> $trait_name for __NongenericClosure<'closure> {
            fn call_once($receiver, $($argument: $argument_type),*) -> $output {
                let ($($capture,)*) = $receiver.0;
                $($once_bindings)*
                $body
            }
            $crate::__generic_closure_if_alloc! {
                fn call_box(
                    self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output {
                    $trait_name::call_once(*self, $($argument),*)
                }
            }
        }
        __NongenericClosure(
            ($($constructor,)*),
            ::core::marker::PhantomData,
        )
    }};
}
