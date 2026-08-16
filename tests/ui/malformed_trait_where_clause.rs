use generic_closure::closure_trait;

closure_trait!(Broken<T: std::fmt::Debug>(value: T) where);

fn main() {}
