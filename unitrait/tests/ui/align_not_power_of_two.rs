unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 3)]
        #[drop_symbol = "_ui_bad_align_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_bad_align_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
