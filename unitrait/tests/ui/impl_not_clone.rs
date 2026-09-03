unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[clone_symbol = "_ui_impl_not_clone_clone"]
        pub type Context: Clone;

        #[symbol = "_ui_impl_not_clone_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

pub struct NotClone(pub u32);

struct MyImpl;

impl FooDriver for MyImpl {
    type Context = NotClone;

    fn new() -> NotClone {
        NotClone(0)
    }
}

foo_impl!(MyImpl);

fn main() {}
