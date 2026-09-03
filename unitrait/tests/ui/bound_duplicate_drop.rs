unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_bound_duplicate_drop_drop"]
        pub type Context: Drop + Drop;

        #[symbol = "_ui_bound_duplicate_drop_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
