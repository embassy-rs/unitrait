unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        pub type Context: Drop;

        #[symbol = "_ui_drop_bound_without_symbol_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
