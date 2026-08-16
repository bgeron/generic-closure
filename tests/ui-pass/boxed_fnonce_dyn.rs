#![deny(warnings)]

use generic_closure::{closure, closure_trait};

closure_trait!(Finish(self, value: String) -> String);

fn main() {
    let prefix = String::from("answer=");
    let finish = closure!(
        prefix: String,
        Finish(self, value: String) -> String { prefix + &value }
    );
    let finish: Box<dyn Finish> = Box::new(finish);

    assert_eq!(finish.call_box(String::from("42")), "answer=42");
}
