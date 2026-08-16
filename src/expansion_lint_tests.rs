// Invoke the macros from inside this crate so rustc and Clippy inspect local
// expansions rather than treating them only as expansions of an external macro.
// The deny levels turn every default warning into a test failure.

#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use core::{cell::Cell, fmt::Debug};
#[cfg(feature = "alloc")]
use std::boxed::Box;
use std::{
    format,
    string::{String, ToString},
    vec::Vec,
};

crate::closure_trait!(pub DescribeForLint<T: Debug>(value: T) -> String);
crate::closure_trait!(pub ObserveForLint<T: Debug>(value: T));
crate::closure_trait!(pub RenderForLint(value: i32) -> String);
crate::closure_trait!(pub RecordForLint(value: i32));
crate::closure_trait!(pub AccumulateForLint<T: Debug>(&mut self, value: T) -> usize);
crate::closure_trait!(pub FinishForLint(self, value: String) -> String);
crate::closure_trait!(pub CombineForLint<T: Debug>(left: T, right: T) -> String);
crate::closure_trait!(pub TickForLint() -> usize);
crate::closure_trait!(WhereForLint<T: Debug>() -> T where ValueForLint: ProvideForLint<T>);

struct ValueForLint;

trait ProvideForLint<T> {
    fn provide() -> T;
}

impl ProvideForLint<i32> for ValueForLint {
    fn provide() -> i32 {
        42
    }
}

#[test]
fn representative_expansions_are_lint_clean() {
    let borrowed = String::from("borrowed");
    let moved = String::from("moved");
    let cloned = String::from("cloned");

    let describe = crate::closure!(
        &borrowed: String,
        moved: String,
        clone cloned: String,
        DescribeForLint<T: Debug>(value: T) -> String {
            format!("{borrowed}/{moved}/{cloned}: {value:?}")
        }
    );

    assert_eq!(cloned, "cloned");
    assert_eq!(describe.call(42), "borrowed/moved/cloned: 42");
    assert_eq!(describe.call(true), "borrowed/moved/cloned: true");

    let referenced_owner = String::from("referenced");
    let referenced = referenced_owner.as_str();
    let describe_reference = crate::closure!(
        referenced: &'closure str,
        DescribeForLint<T: Debug>(value: T) -> String {
            format!("{referenced}: {value:?}")
        }
    );
    assert_eq!(describe_reference.call(42), "referenced: 42");

    let calls = Cell::new(0);
    let observe = crate::closure!(
        &calls: Cell<usize>,
        ObserveForLint<T: Debug>(_value: T) {
            calls.set(calls.get() + 1);
        }
    );

    observe.call(42);
    observe.call("hello");
    assert_eq!(calls.get(), 2);

    let prefix = String::from("value=");
    let render = crate::closure!(
        prefix: String,
        RenderForLint(value: i32) -> String { format!("{prefix}{value}") }
    );
    let render: &dyn RenderForLint = &render;
    assert_eq!(render.call(42), "value=42");

    let total = Cell::new(0);
    let record = crate::closure!(
        &total: Cell<i32>,
        RecordForLint(value: i32) { total.set(total.get() + value); }
    );
    record.call(42);
    assert_eq!(total.get(), 42);

    let values = Vec::new();
    let cloned_values = Vec::new();
    let mut accumulate = crate::closure!(
        mut values: Vec<String>,
        clone mut cloned_values: Vec<String>,
        AccumulateForLint<T: Debug>(&mut self, value: T) -> usize {
            values.push(format!("{value:?}"));
            cloned_values.push(values.len().to_string());
            values.len() + cloned_values.len()
        }
    );
    assert_eq!(accumulate.call_mut(42), 2);
    assert_eq!(accumulate.call_once(true), 4);
    assert!(cloned_values.is_empty());

    let prefix = String::from("finished=");
    let finish = crate::closure!(
        prefix: String,
        FinishForLint(self, value: String) -> String { prefix + &value }
    );
    #[cfg(feature = "alloc")]
    {
        let finish: Box<dyn FinishForLint> = Box::new(finish);
        assert_eq!(finish.call_box(String::from("42")), "finished=42");
    }
    #[cfg(not(feature = "alloc"))]
    assert_eq!(finish.call_once(String::from("42")), "finished=42");

    let combine = crate::closure!(
        CombineForLint<T: Debug>(left: T, right: T) -> String {
            format!("{left:?}/{right:?}")
        }
    );
    assert_eq!(combine.call(20, 22), "20/22");

    let tick = crate::closure!(TickForLint() -> usize { 42 });
    assert_eq!(tick.call(), 42);

    let with_where = crate::closure!(
        WhereForLint<T: Debug>() -> T
        where ValueForLint: ProvideForLint<T>
        {
            <ValueForLint as ProvideForLint<T>>::provide()
        }
    );
    assert_eq!(with_where.call::<i32>(), 42);
}
