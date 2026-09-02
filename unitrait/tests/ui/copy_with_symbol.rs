unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_copy_with_symbol_drop"]
        pub type Context: Copy;

        #[symbol = "_ui_copy_with_symbol_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
