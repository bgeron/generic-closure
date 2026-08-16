use generic_closure::{closure, closure_trait};
use std::fmt::{Debug, Display};

closure_trait!(Describe<T: Debug>(value: T) -> String);

fn main() {
    let _ = closure!(
        Describe<T: Display>(value: T) -> String { value.to_string() }
    );
}
