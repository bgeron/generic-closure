#![allow(unused_variables)]

use generic_closure::{closure, closure_trait};

closure_trait!(Add(left: i32, right: i32) -> i32);

fn main() {
    let offset = 1;
    let right = 40;
    let _add = closure!(
        offset: i32,
        right: i32,
        Add(left: i32, right: i32) -> i32 { left + *right + *offset }
    );
}
