unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[opaque(size = 16, align = 8)]
        #[symbol = "_ui_dup_opaque_drop"]
        pub type Context;

        #[symbol = "_ui_dup_opaque_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
