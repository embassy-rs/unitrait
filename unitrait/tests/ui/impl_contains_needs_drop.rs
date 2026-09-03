unitrait::unitrait! {
    pub trait FooDriver {
        // No `Drop` bound, so the implementation's associated type must not need drop, not
        // even through a field.
        #[opaque(size = 8, align = 4)]
        pub type Context;

        #[symbol = "_ui_impl_contains_needs_drop_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

pub struct Noisy(pub u32);

impl Drop for Noisy {
    fn drop(&mut self) {}
}

pub struct Wrapper(pub Noisy);

struct MyImpl;

impl FooDriver for MyImpl {
    type Context = Wrapper;

    fn new() -> Wrapper {
        Wrapper(Noisy(0))
    }
}

foo_impl!(MyImpl);

fn main() {}
