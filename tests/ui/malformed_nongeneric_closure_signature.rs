use generic_closure::closure;

fn main() {
    let _ = closure!(
        Render(i32) -> String { String::new() }
    );
}
