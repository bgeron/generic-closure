mod interface {
    use generic_closure::closure_trait;

    closure_trait!(Hidden(value: i32) -> i32);
}

fn require_hidden<T: interface::Hidden>() {}

fn main() {}
