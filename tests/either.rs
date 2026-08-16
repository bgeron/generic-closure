#![cfg(feature = "either")]

use generic_closure::{Either, closure, closure_trait};
use std::fmt::Display;

closure_trait!(Render<T: Display>(value: T) -> String);

fn renderer(use_left_body: bool) -> impl Render {
    let prefix = String::from("left: ");
    let left = closure!(
        prefix: String,
        Render<T: Display>(value: T) -> String { format!("{prefix}{value}") }
    );

    let suffix = String::from(" right");
    let right = closure!(
        suffix: String,
        Render<T: Display>(value: T) -> String { format!("{value}{suffix}") }
    );

    if use_left_body {
        Either::Left(left)
    } else {
        Either::Right(right)
    }
}

#[test]
fn either_unifies_two_polymorphic_closure_bodies() {
    // Both values have the same opaque `Either<_, _>` type despite selecting
    // different generated closure bodies, so an array can hold them together.
    let renderers = [renderer(true), renderer(false)];

    assert_eq!(renderers[0].call(42), "left: 42");
    assert_eq!(renderers[0].call(1.5), "left: 1.5");
    assert_eq!(renderers[1].call(42), "42 right");
    assert_eq!(renderers[1].call(1.5), "1.5 right");
}

fn nested_renderer(body: u8) -> impl Render {
    let a = closure!(
        Render<T: Display>(value: T) -> String { format!("a: {value}") }
    );
    let b = closure!(
        Render<T: Display>(value: T) -> String { format!("b: {value}") }
    );
    let c = closure!(
        Render<T: Display>(value: T) -> String { format!("c: {value}") }
    );

    match body {
        0 => Either::Left(a),
        1 => Either::Right(Either::Left(b)),
        _ => Either::Right(Either::Right(c)),
    }
}

#[test]
fn either_composes_recursively_for_more_than_two_bodies() {
    assert_eq!(nested_renderer(0).call(10), "a: 10");
    assert_eq!(nested_renderer(1).call(2.5), "b: 2.5");
    assert_eq!(nested_renderer(2).call("hello"), "c: hello");
}

mod trait_name_collisions {
    use generic_closure::closure_trait;

    closure_trait!(Left(value: i32) -> i32);
    closure_trait!(Right(value: i32) -> i32);
    closure_trait!(Either(value: i32) -> i32);

    pub(super) struct Body;

    impl Left for Body {
        fn call(&self, value: i32) -> i32 {
            value + 1
        }
    }

    impl Right for Body {
        fn call(&self, value: i32) -> i32 {
            value + 2
        }
    }

    impl Either for Body {
        fn call(&self, value: i32) -> i32 {
            value + 3
        }
    }

    pub(super) fn verify() {
        let body = generic_closure::Either::<Body, Body>::Left(Body);
        assert_eq!(Left::call(&body, 39), 40);
        assert_eq!(Right::call(&body, 39), 41);
        assert_eq!(Either::call(&body, 39), 42);
    }
}

mod signature_name_collisions {
    use generic_closure::closure_trait;

    pub(super) struct Left(i32);
    pub(super) struct Right(i32);
    pub(super) struct Either(i32);

    closure_trait!(Sum(left: Left, right: Right, either: Either) -> i32);

    struct Body;

    impl Sum for Body {
        fn call(&self, left: Left, right: Right, either: Either) -> i32 {
            left.0 + right.0 + either.0
        }
    }

    pub(super) fn verify() {
        let body = generic_closure::Either::<Body, Body>::Right(Body);
        assert_eq!(Sum::call(&body, Left(10), Right(20), Either(12)), 42);
    }
}

mod type_parameter_name_collisions {
    use generic_closure::closure_trait;
    use std::fmt::Display;

    closure_trait!(ShowLeft<Left: Display>(value: Left) -> String);
    closure_trait!(ShowRight<Right: Display>(value: Right) -> String);
    closure_trait!(ShowEither<Either: Display>(value: Either) -> String);

    struct Body;

    impl ShowLeft for Body {
        fn call<Left: Display>(&self, value: Left) -> String {
            value.to_string()
        }
    }

    impl ShowRight for Body {
        fn call<Right: Display>(&self, value: Right) -> String {
            value.to_string()
        }
    }

    impl ShowEither for Body {
        fn call<Either: Display>(&self, value: Either) -> String {
            value.to_string()
        }
    }

    pub(super) fn verify() {
        let body = generic_closure::Either::<Body, Body>::Left(Body);
        assert_eq!(ShowLeft::call(&body, 40), "40");
        assert_eq!(ShowRight::call(&body, 41), "41");
        assert_eq!(ShowEither::call(&body, 42), "42");
    }
}

#[test]
fn either_impl_does_not_reserve_public_names() {
    trait_name_collisions::verify();
    signature_name_collisions::verify();
    type_parameter_name_collisions::verify();
}
