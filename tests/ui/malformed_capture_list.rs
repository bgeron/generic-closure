use generic_closure::closure;

fn main() {
    let prefix = String::from("value: ");
    let _ = closure!(
        prefix: String
        Render<T: std::fmt::Display>(value: T) -> String { format!("{prefix}{value}") }
    );
}
