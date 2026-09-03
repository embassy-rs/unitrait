unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[opaque(size = 16, align = 8)]
        #[drop_symbol = "_ui_dup_opaque_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_dup_opaque_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
