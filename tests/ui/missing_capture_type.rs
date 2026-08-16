use generic_closure::closure;

fn main() {
    let prefix = String::from("value=");
    let _ = closure!(
        prefix:,
        Render(value: i32) -> String { format!("{prefix}{value}") }
    );
}
