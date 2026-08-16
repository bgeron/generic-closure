#![deny(unused_doc_comments)]

use generic_closure::closure_trait;
use std::fmt::Display;

closure_trait!(
    /// Converts displayable values to strings.
    ///
    /// This second line verifies that multiple documentation comments survive.
    pub Generic<T: Display>(value: T) -> String
);
closure_trait!(
    /// Doubles an integer.
    pub Nongeneric(value: i32) -> i32
);

fn main() {}
