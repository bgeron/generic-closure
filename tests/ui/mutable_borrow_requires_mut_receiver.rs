use generic_closure::{closure, closure_trait};

closure_trait!(Read(value: i32) -> i32);

fn main() {
    let mut total = 0;
    let _read = closure!(
        &mut total: i32,
        Read(value: i32) -> i32 {
            *total += value;
            *total
        }
    );
}
