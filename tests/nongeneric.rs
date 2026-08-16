use generic_closure::{closure, closure_trait};
use std::cell::Cell;

closure_trait!(Render(value: i32) -> String);
closure_trait!(Observe(value: i32));

#[test]
fn nongeneric_closures_can_be_erased_as_trait_objects() {
    let prefix = String::from("value=");
    let labeled = closure!(
        prefix: String,
        Render(value: i32) -> String { format!("{prefix}{value}") }
    );

    let multiplier = 3;
    let multiplied = closure!(
        multiplier: i32,
        Render(value: i32) -> String { (value * multiplier).to_string() }
    );

    let renderers: Vec<Box<dyn Render>> = vec![Box::new(labeled), Box::new(multiplied)];
    assert_eq!(renderers[0].call(14), "value=14");
    assert_eq!(renderers[1].call(14), "42");
}

fn render_with_prefix(prefix: &str) -> impl Render + '_ {
    closure!(
        prefix: &'closure str,
        Render(value: i32) -> String {
            let _: &str = prefix;
            format!("{prefix}{value}")
        }
    )
}

#[test]
fn nongeneric_closures_can_store_existing_references() {
    let owned_prefix = String::from("referenced=");
    let render = render_with_prefix(owned_prefix.as_str());

    assert_eq!(render.call(42), "referenced=42");
}

#[test]
fn nongeneric_closure_trait_objects_can_borrow_captures() {
    let prefix = String::from("borrowed=");
    let render = closure!(
        &prefix: String,
        Render(value: i32) -> String { format!("{prefix}{value}") }
    );
    let erased: &dyn Render = &render;

    assert_eq!(erased.call(42), "borrowed=42");
}

#[test]
fn nongeneric_unit_output_can_be_omitted() {
    let total = Cell::new(0);
    let observe = closure!(
        &total: Cell<i32>,
        Observe(value: i32) { total.set(total.get() + value); }
    );

    observe.call(20);
    observe.call(22);
    assert_eq!(total.get(), 42);
}

#[cfg(feature = "either")]
#[test]
fn either_unifies_nongeneric_closure_bodies() {
    use generic_closure::Either;

    fn renderer(labeled: bool) -> impl Render {
        let label = String::from("value=");
        if labeled {
            Either::Left(closure!(
                label: String,
                Render(value: i32) -> String { format!("{label}{value}") }
            ))
        } else {
            Either::Right(closure!(
                Render(value: i32) -> String { (value * 2).to_string() }
            ))
        }
    }

    let renderers = [renderer(true), renderer(false)];
    assert_eq!(renderers[0].call(21), "value=21");
    assert_eq!(renderers[1].call(21), "42");
}
