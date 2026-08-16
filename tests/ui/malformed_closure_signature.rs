use generic_closure::closure;

fn main() {
    let _ = closure!(
        Render<T: std::fmt::Display>(T) -> String { String::new() }
    );
}
