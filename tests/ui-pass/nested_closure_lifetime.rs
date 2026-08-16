#![deny(warnings)]

use generic_closure::{closure, closure_trait};
use std::fmt::Debug;

closure_trait!(Describe<T: Debug>(value: T) -> String);

fn make_describer(parts: Vec<&str>) -> impl Describe + '_ {
    closure!(
        clone parts: Vec<&'closure str>,
        Describe<T: Debug>(value: T) -> String {
            format!("{}: {value:?}", parts.join("/"))
        }
    )
}

fn main() {
    let first = String::from("one");
    let second = String::from("two");
    let describe = make_describer(vec![first.as_str(), second.as_str()]);
    assert_eq!(describe.call(42), "one/two: 42");
}
