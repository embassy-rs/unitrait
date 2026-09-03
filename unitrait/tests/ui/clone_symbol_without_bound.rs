unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[clone_symbol = "_ui_clone_symbol_without_bound_clone"]
        pub type Context;

        #[symbol = "_ui_clone_symbol_without_bound_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
