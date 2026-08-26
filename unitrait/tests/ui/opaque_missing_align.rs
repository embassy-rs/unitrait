unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8)]
        #[symbol = "_ui_opaque_missing_align_drop"]
        pub type Context;

        #[symbol = "_ui_opaque_missing_align_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
