use generic_closure::closure;

fn main() {
    let _ = closure!(Broken<T: std::fmt::Debug>(value: T) where { value });
}
