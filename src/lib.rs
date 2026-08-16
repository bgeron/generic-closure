#![doc = include_str!("crate.md")]
#![doc(test(attr(deny(warnings))))]
#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(test)]
extern crate std;

/// The boxed receiver type used by macro-generated `call_box` methods.
#[doc(hidden)]
#[cfg(feature = "alloc")]
pub use alloc::boxed::Box as __Box;

/// A two-variant sum type used to give different closure bodies one concrete type.
///
/// This is re-exported when the default `either` feature is enabled.
#[cfg(feature = "either")]
pub use either::Either;

mod closure;
mod closure_trait;

/// Emits macro-generated items that require the optional `alloc` crate.
#[doc(hidden)]
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! __generic_closure_if_alloc {
    ($($tokens:tt)*) => { $($tokens)* };
}

#[doc(hidden)]
#[cfg(not(feature = "alloc"))]
#[macro_export]
macro_rules! __generic_closure_if_alloc {
    ($($tokens:tt)*) => {};
}

#[doc(hidden)]
#[cfg(feature = "either")]
#[macro_export]
macro_rules! __generic_closure_impl_either {
    (ref, $trait_name:ident, $type_parameter:ident, [$($bounds:tt)+],
        [$($predicates:tt)*], [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        impl<__GenericClosureEitherLeft, __GenericClosureEitherRight> $trait_name
            for $crate::Either<__GenericClosureEitherLeft, __GenericClosureEitherRight>
        where
            __GenericClosureEitherLeft: $trait_name,
            __GenericClosureEitherRight: $trait_name,
        {
            fn call<$type_parameter>(&self, $($argument: $argument_type),*) -> $output
            where
                $type_parameter: $($bounds)+,
                $($predicates)*
            {
                match self {
                    $crate::Either::Left(left) => {
                        $trait_name::call(left, $($argument),*)
                    }
                    $crate::Either::Right(right) => {
                        $trait_name::call(right, $($argument),*)
                    }
                }
            }
        }
    };

    (mut, $trait_name:ident, $type_parameter:ident, [$($bounds:tt)+],
        [$($predicates:tt)*], [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        impl<__GenericClosureEitherLeft, __GenericClosureEitherRight> $trait_name
            for $crate::Either<__GenericClosureEitherLeft, __GenericClosureEitherRight>
        where
            __GenericClosureEitherLeft: $trait_name,
            __GenericClosureEitherRight: $trait_name,
        {
            fn call_mut<$type_parameter>(&mut self, $($argument: $argument_type),*) -> $output
            where
                $type_parameter: $($bounds)+,
                $($predicates)*
            {
                match self {
                    $crate::Either::Left(left) => {
                        $trait_name::call_mut(left, $($argument),*)
                    }
                    $crate::Either::Right(right) => {
                        $trait_name::call_mut(right, $($argument),*)
                    }
                }
            }
        }
    };

    (once, $trait_name:ident, $type_parameter:ident, [$($bounds:tt)+],
        [$($predicates:tt)*], [$($argument:ident: $argument_type:ty),*], $output:ty
    ) => {
        impl<__GenericClosureEitherLeft, __GenericClosureEitherRight> $trait_name
            for $crate::Either<__GenericClosureEitherLeft, __GenericClosureEitherRight>
        where
            __GenericClosureEitherLeft: $trait_name,
            __GenericClosureEitherRight: $trait_name,
        {
            fn call_once<$type_parameter>(self, $($argument: $argument_type),*) -> $output
            where
                $type_parameter: $($bounds)+,
                $($predicates)*
            {
                match self {
                    $crate::Either::Left(left) => {
                        $trait_name::call_once(left, $($argument),*)
                    }
                    $crate::Either::Right(right) => {
                        $trait_name::call_once(right, $($argument),*)
                    }
                }
            }

            $crate::__generic_closure_if_alloc! {
                fn call_box<$type_parameter>(
                    self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output
                where
                    $type_parameter: $($bounds)+,
                    $($predicates)*
                {
                    $trait_name::call_once(*self, $($argument),*)
                }
            }
        }
    };
}

#[doc(hidden)]
#[cfg(not(feature = "either"))]
#[macro_export]
macro_rules! __generic_closure_impl_either {
    ($($tokens:tt)*) => {};
}

#[doc(hidden)]
#[cfg(feature = "either")]
#[macro_export]
macro_rules! __generic_closure_impl_either_nongeneric {
    (ref, $trait_name:ident, [$($argument:ident: $argument_type:ty),*], $output:ty) => {
        impl<__GenericClosureEitherLeft, __GenericClosureEitherRight> $trait_name
            for $crate::Either<__GenericClosureEitherLeft, __GenericClosureEitherRight>
        where
            __GenericClosureEitherLeft: $trait_name,
            __GenericClosureEitherRight: $trait_name,
        {
            fn call(&self, $($argument: $argument_type),*) -> $output {
                match self {
                    $crate::Either::Left(left) => {
                        $trait_name::call(left, $($argument),*)
                    }
                    $crate::Either::Right(right) => {
                        $trait_name::call(right, $($argument),*)
                    }
                }
            }
        }
    };

    (mut, $trait_name:ident, [$($argument:ident: $argument_type:ty),*], $output:ty) => {
        impl<__GenericClosureEitherLeft, __GenericClosureEitherRight> $trait_name
            for $crate::Either<__GenericClosureEitherLeft, __GenericClosureEitherRight>
        where
            __GenericClosureEitherLeft: $trait_name,
            __GenericClosureEitherRight: $trait_name,
        {
            fn call_mut(&mut self, $($argument: $argument_type),*) -> $output {
                match self {
                    $crate::Either::Left(left) => {
                        $trait_name::call_mut(left, $($argument),*)
                    }
                    $crate::Either::Right(right) => {
                        $trait_name::call_mut(right, $($argument),*)
                    }
                }
            }
        }
    };

    (once, $trait_name:ident, [$($argument:ident: $argument_type:ty),*], $output:ty) => {
        impl<__GenericClosureEitherLeft, __GenericClosureEitherRight> $trait_name
            for $crate::Either<__GenericClosureEitherLeft, __GenericClosureEitherRight>
        where
            __GenericClosureEitherLeft: $trait_name,
            __GenericClosureEitherRight: $trait_name,
        {
            fn call_once(self, $($argument: $argument_type),*) -> $output {
                match self {
                    $crate::Either::Left(left) => {
                        $trait_name::call_once(left, $($argument),*)
                    }
                    $crate::Either::Right(right) => {
                        $trait_name::call_once(right, $($argument),*)
                    }
                }
            }

            $crate::__generic_closure_if_alloc! {
                fn call_box(
                    self: $crate::__Box<Self>, $($argument: $argument_type),*
                ) -> $output {
                    $trait_name::call_once(*self, $($argument),*)
                }
            }
        }
    };
}

#[doc(hidden)]
#[cfg(not(feature = "either"))]
#[macro_export]
macro_rules! __generic_closure_impl_either_nongeneric {
    ($($tokens:tt)*) => {};
}

#[cfg(test)]
mod expansion_lint_tests;
