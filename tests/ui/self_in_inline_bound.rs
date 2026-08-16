use generic_closure::closure_trait;

trait Accepts<T: ?Sized> {}

impl<T, U: ?Sized> Accepts<U> for T {}

closure_trait!(Call<T: Accepts<Self>>(value: T));

fn main() {}
