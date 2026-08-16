#![deny(warnings)]

use generic_closure::{closure, closure_trait};

closure_trait!(Render(value: i32) -> String);

fn main() {
    let prefix = String::from("value=");
    let labeled = closure!(
        prefix: String,
        Render(value: i32) -> String { format!("{prefix}{value}") }
    );
    let doubled = closure!(
        Render(value: i32) -> String { (value * 2).to_string() }
    );

    let renderers: Vec<Box<dyn Render>> = vec![Box::new(labeled), Box::new(doubled)];
    assert_eq!(renderers[0].call(21), "value=21");
    assert_eq!(renderers[1].call(21), "42");
}
