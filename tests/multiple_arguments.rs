use generic_closure::{closure, closure_trait};
use std::{cell::Cell, fmt::Display};

closure_trait!(Combine<T: Display>(left: T, separator: &'static str, right: T) -> String);
closure_trait!(Add(left: i32, right: i32) -> i32);
closure_trait!(Accumulate(&mut self, left: i32, right: i32) -> i32);
closure_trait!(Join(self, left: String, right: String) -> String);
closure_trait!(Tick() -> usize);
closure_trait!(Bump(&mut self,) -> usize);
closure_trait!(Take(self) -> String);
closure_trait!(MakeDefault<T: Default>() -> T);

#[test]
fn generic_fn_accepts_multiple_arguments_and_trailing_commas() {
    let prefix = String::from("result=");
    let mut combine = closure!(
        prefix: String,
        Combine<T: Display>(
            left: T,
            separator: &'static str,
            right: T,
        ) -> String {
            format!("{prefix}{left}{separator}{right}")
        }
    );

    assert_eq!(combine.call(20, " + ", 22), "result=20 + 22");
    assert_eq!(combine.call_mut("left", "/", "right"), "result=left/right");
    assert_eq!(combine.call_once(4.0, " + ", 0.2), "result=4 + 0.2");
}

#[test]
fn nongeneric_multiple_arguments_remain_dyn_compatible() {
    let add = closure!(Add(left: i32, right: i32) -> i32 { left + right });
    let add: Box<dyn Add> = Box::new(add);

    assert_eq!(add.call(20, 22), 42);
}

#[test]
fn fn_mut_and_fn_once_forward_every_argument() {
    let total = 0;
    let mut accumulate = closure!(
        mut total: i32,
        Accumulate(&mut self, left: i32, right: i32) -> i32 {
            *total += left + right;
            *total
        }
    );

    assert_eq!(accumulate.call_mut(10, 11), 21);
    assert_eq!(accumulate.call_once(10, 11), 42);

    let separator = String::from("/");
    let join = closure!(
        separator: String,
        Join(self, left: String, right: String) -> String {
            left + &separator + &right
        }
    );
    #[cfg(feature = "alloc")]
    {
        let join: Box<dyn Join> = Box::new(join);
        assert_eq!(
            join.call_box(String::from("left"), String::from("right")),
            "left/right"
        );
    }
    #[cfg(not(feature = "alloc"))]
    assert_eq!(
        join.call_once(String::from("left"), String::from("right")),
        "left/right"
    );
}

#[test]
fn zero_argument_closures_are_supported() {
    let calls = Cell::new(0);
    let tick = closure!(
        &calls: Cell<usize>,
        Tick() -> usize {
            calls.set(calls.get() + 1);
            calls.get()
        }
    );

    assert_eq!(tick.call(), 1);
    assert_eq!(tick.call(), 2);

    let value = 40;
    let mut bump = closure!(
        mut value: usize,
        Bump(&mut self,) -> usize {
            *value += 1;
            *value
        }
    );
    assert_eq!(bump.call_mut(), 41);
    assert_eq!(bump.call_once(), 42);

    let value = String::from("taken");
    let take = closure!(value: String, Take(self) -> String { value });
    #[cfg(feature = "alloc")]
    {
        let take: Box<dyn Take> = Box::new(take);
        assert_eq!(take.call_box(), "taken");
    }
    #[cfg(not(feature = "alloc"))]
    assert_eq!(take.call_once(), "taken");

    let make_default = closure!(MakeDefault<T: Default>() -> T { T::default() });
    assert_eq!(make_default.call::<i32>(), 0);
}

#[cfg(feature = "either")]
#[test]
fn either_forwards_multiple_arguments() {
    use generic_closure::Either;

    fn operation(add: bool) -> impl Add {
        let add_body = closure!(Add(left: i32, right: i32) -> i32 { left + right });
        let multiply_body = closure!(Add(left: i32, right: i32) -> i32 { left * right });
        if add {
            Either::Left(add_body)
        } else {
            Either::Right(multiply_body)
        }
    }

    assert_eq!(operation(true).call(20, 22), 42);
    assert_eq!(operation(false).call(6, 7), 42);

    fn combiner(reverse: bool) -> impl Combine {
        let forward = closure!(
            Combine<T: Display>(left: T, separator: &'static str, right: T) -> String {
                format!("{left}{separator}{right}")
            }
        );
        let reverse_body = closure!(
            Combine<T: Display>(left: T, separator: &'static str, right: T) -> String {
                format!("{right}{separator}{left}")
            }
        );
        if reverse {
            Either::Right(reverse_body)
        } else {
            Either::Left(forward)
        }
    }

    assert_eq!(combiner(false).call(20, "+", 22), "20+22");
    assert_eq!(combiner(true).call("left", "/", "right"), "right/left");
}
