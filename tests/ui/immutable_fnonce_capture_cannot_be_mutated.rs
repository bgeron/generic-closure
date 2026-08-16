use generic_closure::{closure, closure_trait};

closure_trait!(ChangeOnce(self, value: ()));

fn main() {
    let text = String::new();
    let change = closure!(
        text: String,
        ChangeOnce(self, _value: ()) {
            text.push('x');
        }
    );
    change.call_once(());
}
