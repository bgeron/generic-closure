use generic_closure::closure_trait;

closure_trait!(Render<T: std::fmt::Display>(T) -> String);

fn main() {}
