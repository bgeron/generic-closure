use generic_closure::{closure, closure_trait};

closure_trait!(Render(value: i32) -> String);

fn main() {
    let text = String::new();
    let _render = closure!(
        mut text: String,
        Render(value: i32) -> String {
            text.push_str(&value.to_string());
            text.clone()
        }
    );
}
