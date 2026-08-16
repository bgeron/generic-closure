use generic_closure::{closure, closure_trait};
use std::fmt::Display;

closure_trait!(Render<T: Display>(value: T) -> String);
closure_trait!(ExplicitRender(&self, value: i32) -> i32);
closure_trait!(Collect<T: Display>(&mut self, value: T) -> String);
closure_trait!(Finish<T: Display>(self, value: T) -> String);
closure_trait!(Add(&mut self, value: i32) -> i32);
closure_trait!(BoxedFinish(self, value: String) -> String);
#[cfg(feature = "either")]
closure_trait!(ChooseMut(&mut self, value: i32) -> i32);
#[cfg(feature = "either")]
closure_trait!(ChooseOnce(self, value: String) -> String);

#[test]
fn fn_supports_the_full_call_hierarchy() {
    let mut explicit = closure!(
        ExplicitRender(&self, value: i32) -> i32 { value * 2 }
    );
    assert_eq!(explicit.call(10), 20);
    assert_eq!(explicit.call_mut(11), 22);
    assert_eq!(explicit.call_once(21), 42);

    let mut render = closure!(
        Render<T: Display>(value: T) -> String { value.to_string() }
    );

    assert_eq!(render.call(1), "1");
    assert_eq!(render.call_mut(2.5), "2.5");
    assert_eq!(render.call_once("three"), "three");
}

#[test]
fn fn_mut_mutates_only_opted_in_owned_and_cloned_captures() {
    let values = vec![String::from("original")];
    let cloned_values = vec![String::from("clone")];
    let label = String::from("items");
    let mut collect = closure!(
        mut values: Vec<String>,
        clone mut cloned_values: Vec<String>,
        label: String,
        Collect<T: Display>(&mut self, value: T) -> String {
            values.push(value.to_string());
            cloned_values.push(values.len().to_string());
            format!("{label}:{}:{}", values.join("/"), cloned_values.join("/"))
        }
    );

    // `cloned_values` is cloned once and remains available to the caller.
    assert_eq!(cloned_values, ["clone"]);

    assert_eq!(collect.call_mut(1), "items:original/1:clone/2");
    assert_eq!(collect.call_once(2), "items:original/1/2:clone/2/3");
}

#[test]
fn fn_once_can_consume_and_mutate_owned_captures() {
    let prefix = String::from("value=");
    let suffix = String::from("!");
    let mut calls = 0;
    let finish = closure!(
        prefix: String,
        clone mut suffix: String,
        &mut calls: usize,
        Finish<T: Display>(self, value: T) -> String {
            suffix.push('!');
            *calls += 1;
            prefix + &value.to_string() + &suffix
        }
    );

    assert_eq!(finish.call_once(42), "value=42!!");
    assert_eq!(suffix, "!");
    assert_eq!(calls, 1);
}

#[test]
fn fn_mut_can_borrow_a_mutable_value() {
    let mut total = 10;
    {
        let mut add = closure!(
            &mut total: i32,
            Add(&mut self, value: i32) -> i32 {
                *total += value;
                *total
            }
        );

        assert_eq!(add.call_mut(20), 30);
        assert_eq!(add.call_once(12), 42);
    }
    assert_eq!(total, 42);
}

fn add_to(total: &mut i32) -> impl Add + '_ {
    closure!(
        total: &'closure mut i32,
        Add(&mut self, value: i32) -> i32 {
            *total += value;
            *total
        }
    )
}

#[test]
fn fn_mut_can_store_an_existing_mutable_reference() {
    let mut total = 40;
    let mut add = add_to(&mut total);
    assert_eq!(add.call_mut(2), 42);
}

#[cfg(feature = "alloc")]
#[test]
fn fn_once_can_be_called_through_a_boxed_trait_object() {
    let prefix = String::from("answer=");
    let finish = closure!(
        prefix: String,
        BoxedFinish(self, value: String) -> String { prefix + &value }
    );
    let finish: Box<dyn BoxedFinish> = Box::new(finish);

    assert_eq!(finish.call_box(String::from("42")), "answer=42");
}

#[cfg(feature = "either")]
#[test]
fn either_forwards_mutable_and_consuming_calls() {
    use generic_closure::Either;

    fn choose_mut(left: bool) -> impl ChooseMut {
        if left {
            let total = 0;
            Either::Left(closure!(
                mut total: i32,
                ChooseMut(&mut self, value: i32) -> i32 {
                    *total += value;
                    *total
                }
            ))
        } else {
            let total = 0;
            Either::Right(closure!(
                mut total: i32,
                ChooseMut(&mut self, value: i32) -> i32 {
                    *total -= value;
                    *total
                }
            ))
        }
    }

    fn choose_once(left: bool) -> impl ChooseOnce {
        if left {
            let prefix = String::from("answer=");
            Either::Left(closure!(
                prefix: String,
                ChooseOnce(self, value: String) -> String { prefix + &value }
            ))
        } else {
            let suffix = String::from("!");
            Either::Right(closure!(
                suffix: String,
                ChooseOnce(self, value: String) -> String { value + &suffix }
            ))
        }
    }

    let mut add = choose_mut(true);
    assert_eq!(add.call_mut(2), 2);
    assert_eq!(add.call_once(40), 42);

    let subtract = choose_mut(false);
    #[cfg(feature = "alloc")]
    {
        let subtract: Box<dyn ChooseMut> = Box::new(subtract);
        assert_eq!(subtract.call_box(2), -2);
    }
    #[cfg(not(feature = "alloc"))]
    assert_eq!(subtract.call_once(2), -2);

    assert_eq!(choose_once(true).call_once(String::from("42")), "answer=42");
    let finish = choose_once(false);
    #[cfg(feature = "alloc")]
    {
        let finish: Box<dyn ChooseOnce> = Box::new(finish);
        assert_eq!(finish.call_box(String::from("42")), "42!");
    }
    #[cfg(not(feature = "alloc"))]
    assert_eq!(finish.call_once(String::from("42")), "42!");
}
