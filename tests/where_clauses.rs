use generic_closure::{closure, closure_trait};
use std::{
    fmt::{Debug, Display},
    ops::Range,
};

struct Values;

trait Provide<T> {
    fn provide() -> T;
}

impl Provide<i32> for Values {
    fn provide() -> i32 {
        42
    }
}

impl Provide<&'static str> for Values {
    fn provide() -> &'static str {
        "forty-two"
    }
}

closure_trait!(
    Make<T: Debug>() -> T
    where
        Values: Provide<T>,
        T: PartialEq,
);

closure_trait!(
    Remember<T: Debug>(&mut self) -> T
    where
        Values: Provide<T>
);

closure_trait!(
    Take<T: Debug>(self) -> T
    where
        Values: Provide<T>
);

closure_trait!(
    Observe<T: Debug>(value: T)
    where
        T: PartialEq,
);

closure_trait!(
    FormatRange<T: Display>(range: Range<T>) -> String
    where
        Range<T>: Iterator<Item = T>
);

#[test]
fn where_clauses_support_reverse_bounds_and_the_call_hierarchy() {
    let mut make = closure!(
        Make<T: Debug>() -> T
        where
            Values: Provide<T>,
            T: PartialEq,
        {
            <Values as Provide<T>>::provide()
        }
    );

    assert_eq!(make.call::<i32>(), 42);
    assert_eq!(make.call_mut::<&'static str>(), "forty-two");
    assert_eq!(make.call_once::<i32>(), 42);
}

#[test]
fn where_clauses_can_constrain_types_derived_from_the_parameter() {
    let format_range = closure!(
        FormatRange<T: Display>(range: Range<T>) -> String
        where
            Range<T>: Iterator<Item = T>
        {
            range
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }
    );

    assert_eq!(format_range.call(1..4), "1, 2, 3");
    assert_eq!(format_range.call('a'..'d'), "a, b, c");
}

#[test]
fn where_clauses_work_for_mutable_and_consuming_closures() {
    let previous = None;
    let mut remember = closure!(
        mut previous: Option<String>,
        Remember<T: Debug>(&mut self) -> T
        where
            Values: Provide<T>
        {
            let value = <Values as Provide<T>>::provide();
            *previous = Some(format!("{value:?}"));
            value
        }
    );

    assert_eq!(remember.call_mut::<i32>(), 42);
    assert_eq!(remember.call_once::<&'static str>(), "forty-two");

    let take = closure!(
        Take<T: Debug>(self) -> T
        where
            Values: Provide<T>
        {
            <Values as Provide<T>>::provide()
        }
    );
    assert_eq!(take.call_once::<i32>(), 42);

    #[cfg(feature = "alloc")]
    {
        let take_boxed = Box::new(closure!(
            Take<T: Debug>(self) -> T
            where
                Values: Provide<T>
            {
                <Values as Provide<T>>::provide()
            }
        ));
        assert_eq!(take_boxed.call_box::<&'static str>(), "forty-two");
    }
}

#[test]
fn where_clauses_work_with_omitted_unit_outputs() {
    let observe = closure!(
        Observe<T: Debug>(value: T)
        where
            T: PartialEq,
        {
            let _ = value == value;
        }
    );

    observe.call(42);
    observe.call("forty-two");
}

#[cfg(feature = "either")]
#[test]
fn either_forwards_calls_with_where_clauses() {
    use generic_closure::Either;

    fn make(left: bool) -> impl Make {
        let left_body = closure!(
            Make<T: Debug>() -> T
            where
                Values: Provide<T>,
                T: PartialEq,
            {
                <Values as Provide<T>>::provide()
            }
        );
        let right_body = closure!(
            Make<T: Debug>() -> T
            where
                Values: Provide<T>,
                T: PartialEq,
            {
                <Values as Provide<T>>::provide()
            }
        );

        if left {
            Either::Left(left_body)
        } else {
            Either::Right(right_body)
        }
    }

    assert_eq!(make(true).call::<i32>(), 42);
    assert_eq!(make(false).call::<&'static str>(), "forty-two");
}
