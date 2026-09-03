unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8)]
        #[drop_symbol = "_ui_opaque_missing_align_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_opaque_missing_align_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
