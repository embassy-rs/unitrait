unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4, frob = 1)]
        #[drop_symbol = "_ui_opaque_unknown_key_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_opaque_unknown_key_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
