use generic_closure::closure_trait;
use std::fmt::Display;

closure_trait!(Render<T: Display>(value: T) -> String);

fn accept_erased(_: &dyn Render) {}

fn main() {}
