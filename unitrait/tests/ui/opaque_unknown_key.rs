unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4, frob = 1)]
        #[drop_symbol = "_ui_opaque_unknown_key_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_opaque_unknown_key_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
