unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[clone_symbol = "_ui_dup_clone_symbol_a"]
        #[clone_symbol = "_ui_dup_clone_symbol_b"]
        pub type Context: Clone;

        #[symbol = "_ui_dup_clone_symbol_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
