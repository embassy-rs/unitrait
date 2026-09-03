unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[clone_symbol = "_ui_dup_clone_symbol_a"]
        #[clone_symbol = "_ui_dup_clone_symbol_b"]
        pub type Context: Clone;

        #[symbol = "_ui_dup_clone_symbol_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
