unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_drop_symbol_without_bound_drop"]
        pub type Context;

        #[symbol = "_ui_drop_symbol_without_bound_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
