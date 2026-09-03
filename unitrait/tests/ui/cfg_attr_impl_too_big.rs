unitrait::unitrait! {
    pub trait FooDriver {
        #[cfg_attr(any(), opaque(size = 64, align = 8))]
        #[opaque(size = 8, align = 4)]
        pub type Context: Copy;

        #[symbol = "_ui_cfg_attr_impl_too_big_new"]
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
