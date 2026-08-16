use generic_closure::{closure, closure_trait};

closure_trait!(Add(left: i32, right: i32) -> i32);

fn main() {
    let _ = closure!(Add(value: i32) -> i32 { value });
}
