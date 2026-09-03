unitrait::unitrait! {
    pub trait FooDriver {
        #[drop_symbol = "_ui_opaque_missing_attr_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_opaque_missing_attr_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
