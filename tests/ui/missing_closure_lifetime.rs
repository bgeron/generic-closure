use generic_closure::{closure, closure_trait};

closure_trait!(Render(value: i32) -> String);

fn make_renderer(prefix: &str) -> impl Render + '_ {
    closure!(
        prefix: &str,
        Render(value: i32) -> String { format!("{prefix}{value}") }
    )
}

fn main() {}
