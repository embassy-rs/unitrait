unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_drop_symbol_without_bound_drop"]
        pub type Context;

        #[symbol = "_ui_drop_symbol_without_bound_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
