unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_impl_too_big_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_impl_too_big_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl FooDriver for MyImpl {
    type Context = [u8; 9];

    fn new() -> [u8; 9] {
        [0; 9]
    }
}

foo_impl!(MyImpl);

fn main() {}
