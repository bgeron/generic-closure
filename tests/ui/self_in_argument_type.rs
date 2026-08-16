use generic_closure::closure_trait;

closure_trait!(Compare<T: Copy>(value: T, other: &Self) -> bool);

fn main() {}
