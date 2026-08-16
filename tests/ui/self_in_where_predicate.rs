use generic_closure::closure_trait;

closure_trait!(Call<T: Copy>(value: T) where Self: Sized);

fn main() {}
