use generic_closure::{closure, closure_trait};

closure_trait!(Render(value: i32) -> String);

fn main() {
    let _ = closure!(
        Render(value: i64) -> String { value.to_string() }
    );
}
