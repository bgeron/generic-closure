# generic-closure

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT or Apache 2.0 licensed][license-badge]][license-url]

[crates-badge]: https://img.shields.io/crates/v/generic-closure.svg
[crates-url]: https://crates.io/crates/generic-closure
[docs-badge]: https://docs.rs/generic-closure/badge.svg
[docs-url]: https://docs.rs/generic-closure
[license-badge]: https://img.shields.io/crates/l/generic-closure.svg
[license-url]: #license

A proof of concept approach to generic closures in Rust.

- Rust doesn't allow closures to be generic.

- You can make generic inner functions `fn f<T>(..)` in a function, but these
  are not allowed to mention any outer variables.

- In fact, in Rust there isn't a first-class type of generic functions.
  Haskell has `forall a. Trait a =>`, but Rust doesn't have `for<T>`.

This trait emulates polymorphic functions using a trait. Define your "polymorphic
function trait" with `closure_trait!`:

```rust
# use std::fmt::Debug;
#
generic_closure::closure_trait!(
    DebuggableToString<T: Debug>(value: T) -> String
);
```

Then create your pseudo-closure with `closure!`:

```rust
# use std::fmt::Debug;
#
# generic_closure::closure_trait!(
#     DebuggableToString<T: Debug>(value: T) -> String
# );
let prefix = "looks like ";
let debug = generic_closure::closure!(
    prefix: &'static str,
    DebuggableToString<T: Debug>(x: T) -> String {
        format!("{prefix}{x:?}")
    }
);
assert_eq!(debug.call(3i32), "looks like 3");
assert_eq!(debug.call(3f32), "looks like 3.0");
```

This expands to roughly:

```rust
# #![allow(dead_code, unused_variables)]
# use std::fmt::Debug;
#
trait DebuggableToString {
    fn call<T: Debug>(&self, value: T) -> String;

    // also call_mut and call_once; call_box with the `alloc` feature
}

# let prefix = "looks like ";
let debug = {
    struct GeneratedClosure {
        prefix: &'static str,
    }

    impl DebuggableToString for GeneratedClosure {
        fn call<T: Debug>(&self, x: T) -> String {
            let prefix = &self.prefix;
            format!("{prefix}{x:?}")
        }
    }

    GeneratedClosure { prefix }
};
```

Trait `DebuggableToString` is not object-safe, because of the generic method `fn call<T: Debug>`. So
you can use multiple closures by [wrapping them with Either](#multiple-function-bodies).

**AI disclosure.** Initial concept, design, and README mostly by me, code and most other docs by AI.

This is a proof of concept — I may change the API in future versions.

I'm not really aware of similar crates / related work, if any exist, please share on GitHub discussions!

This crate supports `no_std` without requiring an allocator. By default, the `alloc` feature is active and it adds the `call_box` method.

**Limitations.** A `closure!` invocation cannot refer to generic parameters or `Self` from an
enclosing item. Argument types, output types, generic bounds, and `where` predicates in
`closure_trait!` cannot refer to `Self`. Apart from documentation comments, trait attributes
are not supported. Expansion-controlling attributes such as `#[cfg(...)]` may precede the
macro invocation; ordinary item attributes placed there do not propagate to the generated
trait.

## References, moving, capture types

If the variables in your context need lifetimes, use `&'closure`:

```rust
# fn main() {
# use std::fmt::Debug;
#
# generic_closure::closure_trait!(
#     DebuggableToString<T: Debug>(value: T) -> String
# );
let prefix : String = format!("looks like ");
let prefix : &str = &prefix;
let debug = generic_closure::closure!(
    prefix: &'closure str,
    DebuggableToString<T: Debug>(x: T) -> String {
        format!("{prefix}{x:?}")
    }
);
assert_eq!(debug.call(3i32), "looks like 3");
# }
```

Of course, such closures are only able to live as long as the variables
they refer to. Essentially you cannot return such closures.

If you need to return your closure, your options are

1. move the value: `prefix: String` — like `move ||` closure syntax
2. clone the value: `clone prefix: String`

```rust
use generic_closure::{closure_trait, closure};
# use std::fmt::Debug;

closure_trait!(
    DebuggableToString<T: Debug>(value: T) -> String
);

fn f() -> impl DebuggableToString {
    let moved : String = "looks ".to_string();
    let cloned : String = "like ".to_string();
    let debug = closure!(
        moved: String,
        clone cloned: String,
        DebuggableToString<T: Debug>(x: T) -> String {
            format!("{moved}{cloned}{x:?}")
        }
    );

    assert_eq!(debug.call(3i32), "looks like 3");
    assert_eq!(cloned, "like ");
    // assert_eq!(moved, "looks ");    <-- impossible: variable has been moved
    debug
}

# f();
```

To opt out of moving, write `&prefix: String`.

```rust
# use generic_closure::{closure_trait, closure};
# use std::fmt::Debug;
#
# closure_trait!(
#     DebuggableToString<T: Debug>(value: T) -> String
# );
#
let prefix : String = "looks like ".to_string();
let debug = closure!(
    &prefix: String, // we prepended &
    DebuggableToString<T: Debug>(x: T) -> String {
        format!("{prefix}{x:?}")
    }
);

assert_eq!(debug.call(3i32), "looks like 3");
assert_eq!(prefix, "looks like ");
```

### Multiple function bodies

`dyn Trait` is incompatible with generic methods. Use `either::Either` instead — which implements your closure trait:

```rust
# #[cfg(feature = "either")]
# {
use generic_closure::{closure_trait, closure, Either};
# use std::fmt::Debug;
#
# closure_trait!(
#     DebuggableToString<T: Debug>(value: T) -> String
# );
# let prefix = "looks like ";
# let debug = closure!(
#     prefix: &'static str,
#     DebuggableToString<T: Debug>(x: T) -> String {
#         format!("{prefix}{x:?}")
#     }
# );
# assert_eq!(debug.call(3i32), "looks like 3");
# assert_eq!(debug.call(3f32), "looks like 3.0");

let closures = [
    Either::Left(debug),
    Either::Right(closure!(DebuggableToString<T: Debug>(x: T) -> String {
        format!("your value's Debug is {} bytes long", format!("{x:?}").len())
    }))
];

let strings : Vec<String> = closures.into_iter().flat_map(|f| [
    f.call(3i32),
    f.call(3f32)
]).collect();

assert_eq!(strings, [
    "looks like 3",
    "looks like 3.0",
    "your value's Debug is 1 bytes long",
    "your value's Debug is 3 bytes long",
]);
# }
```

Just put `Either::Left` and `Either::Right` around your different kinds of closures. Here, each element of `closures` is an `Either<(anonymous closure struct #1), (anonymous closure struct #2)>`.

If you have more than 2 invocations of `closure!`, you can make a bigger tree, e.g. `Either::Left(..)` / `Either::Right(Either::Left(..))` / `Either::Right(Either::Right(..))`.

`either` is a default-enabled feature flag of this crate.

## Fn, FnMut, and FnOnce

An omitted receiver is shorthand for `&self` and creates an Fn-like trait. Write
the receiver explicitly to select FnMut or FnOnce behavior:

```rust
# use generic_closure::{closure, closure_trait};
# use std::fmt::Display;
#
closure_trait!(Collect<T: Display>(&mut self, value: T) -> usize);

let values = Vec::new();
let mut collect = closure!(
    mut values: Vec<String>,
    Collect<T: Display>(&mut self, value: T) -> usize {
        values.push(value.to_string());
        values.len()
    }
);

assert_eq!(collect.call_mut(1), 1);
assert_eq!(collect.call_mut("two"), 2);
// The final call consumes `collect`.
assert_eq!(collect.call_once(3.0), 3);
```

Mutable access to an owned or cloned capture is opt-in:

- `mut value: Type` moves a mutable capture;
- `clone mut value: Type` clones a mutable capture;
- `&mut value: Type` borrows a mutable variable from the context;
- `value: &'closure mut Type` stores an existing mutable reference.

Without `mut`, an owned FnMut capture remains `&Type` in the body rather than
`&mut Type`. Mutable captures require an `&mut self` or `self` signature; using
them with an Fn-like `&self` signature is an error.

An FnOnce-like closure receives its owned captures by value, so its body can
consume them:

```rust
# use generic_closure::{closure, closure_trait};
#
closure_trait!(Finish(self, value: String) -> String);

let prefix = String::from("answer=");
let finish = closure!(
    prefix: String,
    Finish(self, value: String) -> String { prefix + &value }
);
assert_eq!(finish.call_once(String::from("42")), "answer=42");
```

This mirrors Rust's standard hierarchy: every `Fn` is also `FnMut` and `FnOnce`,
and every `FnMut` is also `FnOnce`.

With the default `alloc` feature, every generated trait also has `call_box`. If
its call method is non-generic, this permits consuming it as a trait object:

```rust
# #[cfg(feature = "alloc")]
# {
use generic_closure::{closure, closure_trait};

closure_trait!(Finish(self, value: String) -> String);
let prefix = String::from("boxed=");
let finish = closure!(
    prefix: String,
    Finish(self, value: String) -> String { prefix + &value }
);
let finish: Box<dyn Finish> = Box::new(finish);

assert_eq!(finish.call_box(String::from("42")), "boxed=42");
# }
```

This is not possible when the call method is generic, because such traits are not
object-safe.

## Example: return type parametricity

A generic closure can be stateful, and maintain state across calls with different result types:

```rust
use generic_closure::{closure, closure_trait};
use std::fmt::Debug;

closure_trait!(
    DefaultAndPrevious<T: Debug + Default>(&mut self) -> (T, Option<String>)
);

let previous = None;
let mut default_and_previous = closure!(
    mut previous: Option<String>,
    DefaultAndPrevious<T: Debug + Default>(&mut self) -> (T, Option<String>) {
        let result = T::default();
        let previous = previous.replace(format!("{result:?}"));
        (result, previous)
    }
);

assert_eq!(default_and_previous.call_mut::<bool>(), (false, None));
assert_eq!(
    default_and_previous.call_mut::<f64>(),
    (0.0, Some("false".to_owned())),
);
assert_eq!(
    default_and_previous.call_mut::<u32>(),
    (0, Some("0.0".to_owned())),
);
```

The closure stores the formatted string rather than the result itself, since
successive calls need not use the same `T`.

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE), or
- [MIT License](LICENSE-MIT)

at your option.
