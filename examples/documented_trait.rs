use generic_closure::closure;
use std::fmt::Display;

pub mod interface {
    use generic_closure::closure_trait;
    use std::fmt::Display;

    closure_trait!(
        /// Converts displayable values to strings.
        pub Render<T: Display>(value: T) -> String
    );
}

use interface::Render;

fn main() {
    let render = closure!(
        Render<T: Display>(value: T) -> String { value.to_string() }
    );
    assert_eq!(render.call(42), "42");
}
