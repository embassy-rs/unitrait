unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        pub type Context: Clone;

        #[symbol = "_ui_clone_bound_without_symbol_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
