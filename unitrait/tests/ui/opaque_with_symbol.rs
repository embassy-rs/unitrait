unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_opaque_with_symbol_drop"]
        pub type Context;

        #[symbol = "_ui_opaque_with_symbol_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
