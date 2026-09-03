use std::rc::Rc;

unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 16, align = 8)]
        #[drop_symbol = "_ui_impl_not_send_drop"]
        pub type Context: Send + Drop;

        #[symbol = "_ui_impl_not_send_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl Foo for MyImpl {
    type Context = Rc<u32>;

    fn new() -> Rc<u32> {
        Rc::new(0)
    }
}

foo_impl!(MyImpl);

fn main() {}
