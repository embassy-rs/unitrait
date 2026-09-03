use std::rc::Rc;

unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 16, align = 8)]
        #[drop_symbol = "_ui_impl_not_send_drop"]
        pub type Context: Send + Drop;

        #[symbol = "_ui_impl_not_send_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl FooDriver for MyImpl {
    type Context = Rc<u32>;

    fn new() -> Rc<u32> {
        Rc::new(0)
    }
}

foo_impl!(MyImpl);

fn main() {}
