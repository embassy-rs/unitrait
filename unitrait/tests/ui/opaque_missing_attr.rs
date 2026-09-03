unitrait::unitrait! {
    pub trait Foo {
        #[drop_symbol = "_ui_opaque_missing_attr_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_opaque_missing_attr_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
