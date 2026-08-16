/// Declares a trait for an Fn-, FnMut-, or FnOnce-like closure.
///
/// # Quick guide
///
/// Declare zero or more named parameters and an optional output:
///
/// ```text
/// closure_trait!(Trait<T: Bounds>(left: T, right: T) -> Output);
/// closure_trait!(Trait(first: Input, second: Input) -> Output);
/// ```
///
/// Put `&self`, `&mut self`, or `self` before the arguments to select Fn,
/// FnMut, or FnOnce behavior. Omitting the receiver means `&self`; omitting the output
/// means `()`. Documentation comments and visibility such as `pub` may precede
/// the trait name. Other trait attributes are not supported. Expansion-controlling
/// attributes such as `#[cfg(...)]` may precede the macro invocation; ordinary
/// item attributes placed there do not propagate to the generated trait. Use the
/// resulting trait name and matching signature with [`closure!`](crate::closure).
///
/// A signature containing `<T: Bounds>` generates generic call methods, so one
/// value can be called with every argument type satisfying those bounds. Without
/// a type parameter, the call methods are object-safe. The receiver selects the
/// closure kind:
///
/// - an omitted receiver, or `&self`, generates `call`, `call_mut`, and
///   `call_once`;
/// - `&mut self` generates `call_mut` and `call_once`;
/// - `self` generates `call_once`.
///
/// Fn- and FnMut-like traits provide their less restrictive call methods by
/// forwarding to the required method. With the default `alloc` feature, every
/// trait also has `call_box`, allowing a non-generic FnOnce-like trait to be
/// called through `Box<dyn Trait>`.
///
/// ```
/// use generic_closure::{closure, closure_trait};
/// use std::fmt::Display;
///
/// closure_trait!(
///     /// Returns the display width of a value.
///     pub DisplayLength<T: Display>(value: T) -> usize
/// );
///
/// fn invoke_twice(f: &impl DisplayLength) {
///     assert_eq!(f.call(123), 3);
///     assert_eq!(f.call("hello"), 5);
/// }
///
/// let display_length = closure!(
///     DisplayLength<T: Display>(value: T) -> usize { value.to_string().len() }
/// );
/// invoke_twice(&display_length);
/// ```
///
/// A non-generic closure trait can erase unrelated bodies behind `dyn Trait`:
///
/// ```
/// use generic_closure::{closure, closure_trait};
///
/// closure_trait!(Render(value: i32) -> String);
/// let render = closure!(
///     Render(value: i32) -> String { format!("value={value}") }
/// );
/// let render: Box<dyn Render> = Box::new(render);
/// assert_eq!(render.call(42), "value=42");
/// ```
///
/// When the `either` feature is enabled, every generated trait is also
/// implemented recursively for `either::Either<L, R>` whenever both `L` and `R`
/// implement it.
///
/// # Syntax diagnostics
///
/// Generic declarations require one type parameter with at least one bound.
/// Arguments use `name: Type` syntax and may be omitted or repeated; a trailing
/// comma is accepted. Generic signatures may append a Rust-like `where` clause
/// for predicates that are not bounds directly on the type parameter. `Self`
/// cannot appear in argument types, the output type, generic bounds, or `where`
/// predicates. Omitting `-> Output` means `-> ()`.
#[macro_export]
macro_rules! closure_trait {
    ($($input:tt)*) => {
        $crate::__generic_closure_parse_trait! {
            @parse_documentation
            documentation = [];
            remaining = [$($input)*];
        }
    };
}

/// Internal parser for [`closure_trait!`].
/// Implementation guide:
///
/// `closure_trait!` separates generic declarations from concrete ones while
/// preserving documentation and visibility. Generic declarations enter this
/// token-tree muncher so nested bound syntax can be consumed up to the argument
/// list; concrete declarations skip directly to signature normalization. The
/// signature arms map an omitted or explicit receiver to the Fn, FnMut, or FnOnce
/// mode. An optional `where` clause is consumed by a second token-tree muncher.
/// `__generic_closure_emit_trait!` then emits the required method, the conventional
/// forwarding methods, the optional `call_box`, and the optional `Either`
/// implementation.
///
/// These helpers are exported so public-macro expansions can resolve them in a
/// downstream crate. Their `__` prefixes and `doc(hidden)` attributes keep them
/// implementation details.
#[doc(hidden)]
#[macro_export]
macro_rules! __generic_closure_parse_trait {
    // Preserve documentation comments, which arrive as `#[doc = "..."]`.
    (@parse_documentation
        documentation = [$($documentation:tt)*];
        remaining = [#[doc = $doc:literal] $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @parse_documentation
            documentation = [$($documentation)* #[doc = $doc]];
            remaining = [$($remaining)*];
        }
    };

    // Non-documentation trait attributes are unsupported. Expansion-controlling
    // attributes such as `cfg` can gate the macro invocation, but ordinary item
    // attributes placed there are not propagated to the generated trait.
    (@parse_documentation
        documentation = [$($documentation:tt)*];
        remaining = [#[$($unsupported:tt)*] $($remaining:tt)*];
    ) => {
        ::core::compile_error!(concat!(
            "closure_trait! accepts only documentation comments before the ",
            "trait name; other trait attributes are unsupported ",
            "(expansion-controlling attributes such as `#[cfg(...)]` may ",
            "precede the macro invocation)"
        ));
    };

    // Generic bounds need recursive parsing because they may contain nested
    // angle brackets.
    (@parse_documentation
        documentation = [$($documentation:tt)*];
        remaining = [
            $visibility:vis $trait_name:ident<$type_parameter:ident: $($remaining:tt)*
        ];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @parse_bounds
            attributes = [$($documentation)*];
            visibility = [$visibility];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            parsed_bounds = [];
            remaining = [$($remaining)*];
        }
    };

    // The tokens in the argument list are parsed separately to distinguish
    // arguments from an explicit receiver followed by arguments.
    (@parse_documentation
        documentation = [$($documentation:tt)*];
        remaining = [
            $visibility:vis $trait_name:ident($($signature:tt)*) -> $output:ty $(;)?
        ];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @emit_nongeneric_signature
            attributes = [$($documentation)*];
            visibility = [$visibility];
            trait_name = [$trait_name];
            signature = [$($signature)*];
            output = [$output];
        }
    };

    (@parse_documentation
        documentation = [$($documentation:tt)*];
        remaining = [
            $visibility:vis $trait_name:ident($($signature:tt)*) $(;)?
        ];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @emit_nongeneric_signature
            attributes = [$($documentation)*];
            visibility = [$visibility];
            trait_name = [$trait_name];
            signature = [$($signature)*];
            output = [()];
        }
    };

    (@parse_documentation $($invalid:tt)*) => {
        ::core::compile_error!(concat!(
            "malformed closure_trait! invocation; expected `Trait<T: Bounds>",
            "(argument: T) -> Output` or `Trait(argument: Input) -> Output`, ",
            "optionally with `&self`, `&mut self`, or `self` before the ",
            "arguments; omit `-> Output` for unit"
        ));
    };

    // Split the final adjacent `>>` in signatures such as
    // `Trait<T: AsRef<str>>(value: T)`. A `>>` inside the bounds is followed by
    // another bound token, whereas the final one is followed by the argument group.
    (@parse_bounds
        attributes = [$($attributes:tt)*];
        visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>>($($signature:tt)*) $($tail:tt)*];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @parse_bounds
            attributes = [$($attributes)*];
            visibility = [$visibility];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            parsed_bounds = [$($parsed_bounds)+ >];
            remaining = [>($($signature)*) $($tail)*];
        }
    };

    (@parse_bounds
        attributes = [$($attributes:tt)*];
        visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>($($signature:tt)*) -> $output:ty where $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @parse_where
            attributes = [$($attributes)*];
            visibility = [$visibility];
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
        attributes = [$($attributes:tt)*];
        visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>($($signature:tt)*) where $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @parse_where
            attributes = [$($attributes)*];
            visibility = [$visibility];
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
        attributes = [$($attributes:tt)*];
        visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>($($signature:tt)*) -> $output:ty $(;)?];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @emit_generic_signature
            attributes = [$($attributes)*];
            visibility = [$visibility];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            bounds = [$($parsed_bounds)+];
            predicates = [];
            signature = [$($signature)*];
            output = [$output];
        }
    };

    (@parse_bounds
        attributes = [$($attributes:tt)*];
        visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)+];
        remaining = [>($($signature:tt)*) $(;)?];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @emit_generic_signature
            attributes = [$($attributes)*];
            visibility = [$visibility];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            bounds = [$($parsed_bounds)+];
            predicates = [];
            signature = [$($signature)*];
            output = [()];
        }
    };

    (@parse_bounds
        attributes = [$($attributes:tt)*];
        visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)*];
        remaining = [$next:tt $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @parse_bounds
            attributes = [$($attributes)*];
            visibility = [$visibility];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            parsed_bounds = [$($parsed_bounds)* $next];
            remaining = [$($remaining)*];
        }
    };

    (@parse_bounds
        attributes = [$($attributes:tt)*];
        visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        parsed_bounds = [$($parsed_bounds:tt)*];
        remaining = [];
    ) => {
        ::core::compile_error!(concat!(
            "malformed closure_trait! generic signature; expected `Trait<T: ",
            "Bounds>(argument: T) -> Output`, optionally with `&self`, ",
            "`&mut self`, or `self` before the arguments; omit `-> Output` ",
            "for unit"
        ));
    };

    // Consume an optional generic method `where` clause up to the invocation's
    // optional semicolon.
    (@parse_where
        attributes = [$($attributes:tt)*];
        visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+];
        signature = [$($signature:tt)*];
        output = [$output:ty];
        parsed_predicates = [$($predicates:tt)+];
        remaining = [$(;)?];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @emit_generic_signature
            attributes = [$($attributes)*];
            visibility = [$visibility];
            trait_name = [$trait_name];
            type_parameter = [$type_parameter];
            bounds = [$($bounds)+];
            predicates = [$($predicates)+];
            signature = [$($signature)*];
            output = [$output];
        }
    };

    (@parse_where
        attributes = [$($attributes:tt)*];
        visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+];
        signature = [$($signature:tt)*];
        output = [$output:ty];
        parsed_predicates = [$($predicates:tt)*];
        remaining = [$next:tt $($remaining:tt)*];
    ) => {
        $crate::__generic_closure_parse_trait! {
            @parse_where
            attributes = [$($attributes)*];
            visibility = [$visibility];
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
            "malformed closure_trait! where clause; expected at least one predicate"
        );
    };

    // Normalize receiver syntax into one of the three closure modes.
    (@emit_generic_signature
        attributes = [$($attributes:tt)*]; visibility = [$visibility:vis];
        trait_name = [$trait_name:ident]; type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+]; predicates = [$($predicates:tt)*];
        signature = [& self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty];
    ) => {
        $crate::__generic_closure_emit_trait! {
            generic ref, [$($attributes)*], [$visibility], $trait_name,
            $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        }
    };
    (@emit_generic_signature
        attributes = [$($attributes:tt)*]; visibility = [$visibility:vis];
        trait_name = [$trait_name:ident]; type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+]; predicates = [$($predicates:tt)*];
        signature = [& mut self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty];
    ) => {
        $crate::__generic_closure_emit_trait! {
            generic mut, [$($attributes)*], [$visibility], $trait_name,
            $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        }
    };
    (@emit_generic_signature
        attributes = [$($attributes:tt)*]; visibility = [$visibility:vis];
        trait_name = [$trait_name:ident]; type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+]; predicates = [$($predicates:tt)*];
        signature = [self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty];
    ) => {
        $crate::__generic_closure_emit_trait! {
            generic once, [$($attributes)*], [$visibility], $trait_name,
            $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        }
    };
    (@emit_generic_signature
        attributes = [$($attributes:tt)*]; visibility = [$visibility:vis];
        trait_name = [$trait_name:ident]; type_parameter = [$type_parameter:ident];
        bounds = [$($bounds:tt)+]; predicates = [$($predicates:tt)*];
        signature = [$($argument:ident: $argument_type:ty),* $(,)?];
        output = [$output:ty];
    ) => {
        $crate::__generic_closure_emit_trait! {
            generic ref, [$($attributes)*], [$visibility], $trait_name,
            $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        }
    };
    (@emit_generic_signature $($invalid:tt)*) => {
        ::core::compile_error!(concat!(
            "malformed closure_trait! generic arguments; expected `name: Type` ",
            "arguments such as `(value: T)` or `(left: T, right: T)`, ",
            "optionally preceded by `&self`, `&mut self`, or `self`"
        ));
    };

    (@emit_nongeneric_signature
        attributes = [$($attributes:tt)*]; visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        signature = [& self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty];
    ) => {
        $crate::__generic_closure_emit_trait! {
            nongeneric ref, [$($attributes)*], [$visibility], $trait_name,
            [$($argument: $argument_type),*], $output
        }
    };
    (@emit_nongeneric_signature
        attributes = [$($attributes:tt)*]; visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        signature = [& mut self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty];
    ) => {
        $crate::__generic_closure_emit_trait! {
            nongeneric mut, [$($attributes)*], [$visibility], $trait_name,
            [$($argument: $argument_type),*], $output
        }
    };
    (@emit_nongeneric_signature
        attributes = [$($attributes:tt)*]; visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        signature = [self $(, $argument:ident: $argument_type:ty)* $(,)?];
        output = [$output:ty];
    ) => {
        $crate::__generic_closure_emit_trait! {
            nongeneric once, [$($attributes)*], [$visibility], $trait_name,
            [$($argument: $argument_type),*], $output
        }
    };
    (@emit_nongeneric_signature
        attributes = [$($attributes:tt)*]; visibility = [$visibility:vis];
        trait_name = [$trait_name:ident];
        signature = [$($argument:ident: $argument_type:ty),* $(,)?];
        output = [$output:ty];
    ) => {
        $crate::__generic_closure_emit_trait! {
            nongeneric ref, [$($attributes)*], [$visibility], $trait_name,
            [$($argument: $argument_type),*], $output
        }
    };
    (@emit_nongeneric_signature $($invalid:tt)*) => {
        ::core::compile_error!(concat!(
            "malformed closure_trait! non-generic arguments; expected `name: ",
            "Type` arguments such as `(value: i32)` or `(left: i32, right: ",
            "i32)`, optionally preceded by `&self`, `&mut self`, or `self`"
        ));
    };
}

/// Rejects `Self` in a declaration by reproducing its signature on a free
/// function, where `Self` is not in scope.
#[doc(hidden)]
#[macro_export]
macro_rules! __generic_closure_validate_declaration {
    (generic $type_parameter:ident, [$($bounds:tt)+], [$($predicates:tt)*],
        [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        const _: () = {
            #[allow(dead_code, unused_variables)]
            fn __generic_closure_validate_declaration<$type_parameter>(
                $($argument: $argument_type),*
            ) -> $output
            where
                $type_parameter: $($bounds)+,
                $($predicates)*
            {
                ::core::unreachable!()
            }
        };
    };

    (nongeneric [$($argument:ident: $argument_type:ty),*], $output:ty) => {
        const _: () = {
            #[allow(dead_code, unused_variables)]
            fn __generic_closure_validate_declaration(
                $($argument: $argument_type),*
            ) -> $output {
                ::core::unreachable!()
            }
        };
    };
}

/// Emits a closure trait after its signature has been parsed.
#[doc(hidden)]
#[macro_export]
macro_rules! __generic_closure_emit_trait {
    (generic ref, [$($attributes:tt)*], [$visibility:vis], $trait_name:ident,
        $type_parameter:ident, [$($bounds:tt)+], [$($predicates:tt)*],
        [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        $crate::__generic_closure_validate_declaration!(
            generic $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        );
        $($attributes)*
        $visibility trait $trait_name {
            fn call<$type_parameter>(&self, $($argument: $argument_type),*) -> $output
            where $type_parameter: $($bounds)+, $($predicates)*;

            fn call_mut<$type_parameter>(&mut self, $($argument: $argument_type),*) -> $output
            where $type_parameter: $($bounds)+, $($predicates)*
            { self.call($($argument),*) }

            #[allow(dead_code)]
            fn call_once<$type_parameter>(mut self, $($argument: $argument_type),*) -> $output
            where Self: Sized, $type_parameter: $($bounds)+, $($predicates)*
            { self.call_mut($($argument),*) }

            $crate::__generic_closure_if_alloc! {
                #[allow(dead_code)]
                fn call_box<$type_parameter>(
                    self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output
                where $type_parameter: $($bounds)+, $($predicates)*
                { self.call($($argument),*) }
            }
        }
        $crate::__generic_closure_impl_either!(
            ref, $trait_name, $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        );
    };

    (generic mut, [$($attributes:tt)*], [$visibility:vis], $trait_name:ident,
        $type_parameter:ident, [$($bounds:tt)+], [$($predicates:tt)*],
        [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        $crate::__generic_closure_validate_declaration!(
            generic $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        );
        $($attributes)*
        $visibility trait $trait_name {
            fn call_mut<$type_parameter>(&mut self, $($argument: $argument_type),*) -> $output
            where $type_parameter: $($bounds)+, $($predicates)*;

            #[allow(dead_code)]
            fn call_once<$type_parameter>(mut self, $($argument: $argument_type),*) -> $output
            where Self: Sized, $type_parameter: $($bounds)+, $($predicates)*
            { self.call_mut($($argument),*) }

            $crate::__generic_closure_if_alloc! {
                #[allow(dead_code)]
                fn call_box<$type_parameter>(
                    mut self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output
                where $type_parameter: $($bounds)+, $($predicates)*
                { self.call_mut($($argument),*) }
            }
        }
        $crate::__generic_closure_impl_either!(
            mut, $trait_name, $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        );
    };

    (generic once, [$($attributes:tt)*], [$visibility:vis], $trait_name:ident,
        $type_parameter:ident, [$($bounds:tt)+], [$($predicates:tt)*],
        [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        $crate::__generic_closure_validate_declaration!(
            generic $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        );
        $($attributes)*
        $visibility trait $trait_name {
            #[allow(dead_code)]
            fn call_once<$type_parameter>(self, $($argument: $argument_type),*) -> $output
            where Self: Sized, $type_parameter: $($bounds)+, $($predicates)*;

            $crate::__generic_closure_if_alloc! {
                #[allow(dead_code)]
                fn call_box<$type_parameter>(
                    self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output
                where $type_parameter: $($bounds)+, $($predicates)*;
            }
        }
        $crate::__generic_closure_impl_either!(
            once, $trait_name, $type_parameter, [$($bounds)+], [$($predicates)*],
            [$($argument: $argument_type),*], $output
        );
    };

    (nongeneric ref, [$($attributes:tt)*], [$visibility:vis], $trait_name:ident,
        [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        $crate::__generic_closure_validate_declaration!(
            nongeneric [$($argument: $argument_type),*], $output
        );
        $($attributes)*
        $visibility trait $trait_name {
            fn call(&self, $($argument: $argument_type),*) -> $output;
            #[allow(dead_code)]
            fn call_mut(&mut self, $($argument: $argument_type),*) -> $output {
                self.call($($argument),*)
            }
            #[allow(dead_code)]
            fn call_once(mut self, $($argument: $argument_type),*) -> $output where Self: Sized {
                self.call_mut($($argument),*)
            }
            $crate::__generic_closure_if_alloc! {
                #[allow(dead_code)]
                fn call_box(
                    self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output {
                    self.call($($argument),*)
                }
            }
        }
        $crate::__generic_closure_impl_either_nongeneric!(
            ref, $trait_name, [$($argument: $argument_type),*], $output
        );
    };

    (nongeneric mut, [$($attributes:tt)*], [$visibility:vis], $trait_name:ident,
        [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        $crate::__generic_closure_validate_declaration!(
            nongeneric [$($argument: $argument_type),*], $output
        );
        $($attributes)*
        $visibility trait $trait_name {
            fn call_mut(&mut self, $($argument: $argument_type),*) -> $output;
            #[allow(dead_code)]
            fn call_once(mut self, $($argument: $argument_type),*) -> $output where Self: Sized {
                self.call_mut($($argument),*)
            }
            $crate::__generic_closure_if_alloc! {
                #[allow(dead_code)]
                fn call_box(
                    mut self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output {
                    self.call_mut($($argument),*)
                }
            }
        }
        $crate::__generic_closure_impl_either_nongeneric!(
            mut, $trait_name, [$($argument: $argument_type),*], $output
        );
    };

    (nongeneric once, [$($attributes:tt)*], [$visibility:vis], $trait_name:ident,
        [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        $crate::__generic_closure_validate_declaration!(
            nongeneric [$($argument: $argument_type),*], $output
        );
        $($attributes)*
        $visibility trait $trait_name {
            #[allow(dead_code)]
            fn call_once(self, $($argument: $argument_type),*) -> $output where Self: Sized;
            $crate::__generic_closure_if_alloc! {
                #[allow(dead_code)]
                fn call_box(
                    self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output;
            }
        }
        $crate::__generic_closure_impl_either_nongeneric!(
            once, $trait_name, [$($argument: $argument_type),*], $output
        );
    };
}
