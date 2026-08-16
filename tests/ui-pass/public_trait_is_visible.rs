mod interface {
    use generic_closure::closure_trait;

    closure_trait!(pub Visible(value: i32) -> i32);
}

struct Body;

impl interface::Visible for Body {
    fn call(&self, value: i32) -> i32 {
        value
    }
}

fn main() {
    assert_eq!(interface::Visible::call(&Body, 42), 42);
}
