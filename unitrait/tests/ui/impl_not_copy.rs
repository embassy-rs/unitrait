unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 32, align = 8)]
        pub type Context: Copy;

        #[symbol = "_ui_impl_not_copy_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl FooDriver for MyImpl {
    type Context = String;

    fn new() -> String {
        String::new()
    }
}

foo_impl!(MyImpl);

fn main() {}
