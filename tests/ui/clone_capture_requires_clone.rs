use generic_closure::{closure, closure_trait};

closure_trait!(Read(value: ()) -> usize);

struct NotClone(usize);

fn main() {
    let value = NotClone(42);
    let _ = closure!(
        clone value: NotClone,
        Read(_input: ()) -> usize { value.0 }
    );
}
