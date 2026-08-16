use generic_closure::{closure, closure_trait};

closure_trait!(Render(value: i32) -> String);

fn main() {
    let render = {
        let prefix = String::from("value=");
        closure!(
            &prefix: String,
            Render(value: i32) -> String { format!("{prefix}{value}") }
        )
    };

    let _ = render.call(42);
}
