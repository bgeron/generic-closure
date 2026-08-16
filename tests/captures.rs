use generic_closure::{closure, closure_trait};
use std::{cell::Cell, cell::RefCell, fmt::Debug, rc::Rc};

closure_trait!(Describe<T: Debug>(value: T) -> String);
closure_trait!(Duplicate<T: Clone>(value: T) -> (T, T));
closure_trait!(Observe<T: Debug>(value: T));

struct CloneCounter {
    text: String,
    clones: Rc<Cell<usize>>,
}

impl Clone for CloneCounter {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);
        Self {
            text: self.text.clone(),
            clones: Rc::clone(&self.clones),
        }
    }
}

#[test]
fn borrow_move_and_clone_capture_at_construction() {
    let borrowed = String::from("borrowed");
    let moved = String::from("moved");
    let clone_count = Rc::new(Cell::new(0));
    let cloned = CloneCounter {
        text: String::from("cloned"),
        clones: Rc::clone(&clone_count),
    };

    let describe = closure!(
        &borrowed: String,
        moved: String,
        clone cloned: CloneCounter,
        Describe<T: Debug>(value: T) -> String {
            format!("{borrowed}, {moved}, {}: {value:?}", cloned.text)
        }
    );

    assert_eq!(clone_count.get(), 1);
    assert_eq!(cloned.text, "cloned");
    assert_eq!(describe.call(42), "borrowed, moved, cloned: 42");
    assert_eq!(describe.call(true), "borrowed, moved, cloned: true");
    assert_eq!(clone_count.get(), 1, "calls must not clone captures");

    drop(describe);
    let mut borrowed = borrowed;
    borrowed.push('!');
    assert_eq!(borrowed, "borrowed!");
}

#[test]
fn omitted_return_type_means_unit() {
    let observed = RefCell::new(Vec::new());
    {
        let observe = closure!(
            &observed: RefCell<Vec<String>>,
            Observe<T: Debug>(value: T) {
                observed.borrow_mut().push(format!("{value:?}"));
            }
        );

        observe.call(42);
        observe.call("hello");
    }
    assert_eq!(observed.into_inner(), ["42", "\"hello\""]);
}

fn describe_parts(parts: Vec<&str>) -> impl Describe + '_ {
    closure!(
        parts: Vec<&'closure str>,
        Describe<T: Debug>(value: T) -> String {
            format!("{}: {value:?}", parts.join("/"))
        }
    )
}

#[test]
fn closure_lifetime_can_appear_nested_in_capture_types() {
    let first = String::from("one");
    let second = String::from("two");
    let describe = describe_parts(vec![first.as_str(), second.as_str()]);

    assert_eq!(describe.call(3), "one/two: 3");
    assert_eq!(describe.call(true), "one/two: true");

    let parts = vec![first.as_str(), second.as_str()];
    let cloned = closure!(
        clone parts: Vec<&'closure str>,
        Describe<T: Debug>(value: T) -> String {
            format!("{}: {value:?}", parts.join("+"))
        }
    );
    assert_eq!(parts, ["one", "two"]);
    assert_eq!(cloned.call(42), "one+two: 42");
}

#[test]
fn generated_lifetime_marker_does_not_reserve_a_capture_name() {
    let __generic_closure_lifetime = String::from("available");
    let describe = closure!(
        __generic_closure_lifetime: String,
        Describe<T: Debug>(value: T) -> String {
            format!("{__generic_closure_lifetime}: {value:?}")
        }
    );

    assert_eq!(describe.call(42), "available: 42");
}

#[test]
fn closure_without_captures_can_return_its_generic_type() {
    let duplicate = closure!(
        Duplicate<T: Clone>(value: T) -> (T, T) { (value.clone(), value) }
    );

    assert_eq!(duplicate.call(7), (7, 7));
    assert_eq!(
        duplicate.call(String::from("hello")),
        (String::from("hello"), String::from("hello"))
    );
}
