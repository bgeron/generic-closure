use generic_closure::{closure, closure_trait};

closure_trait!(Change(&mut self, value: ()));

fn main() {
    let moved = String::new();
    let _moved = closure!(
        moved: String,
        Change(&mut self, _value: ()) {
            moved.push('x');
        }
    );

    let cloned = String::new();
    let _cloned = closure!(
        clone cloned: String,
        Change(&mut self, _value: ()) {
            cloned.push('x');
        }
    );
}
