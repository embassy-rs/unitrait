unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_dup_drop_symbol_a"]
        #[drop_symbol = "_ui_dup_drop_symbol_b"]
        pub type Context: Drop;

        #[symbol = "_ui_dup_drop_symbol_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
