unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4, frob = 1)]
        #[symbol = "_ui_opaque_unknown_key_drop"]
        pub type Context;

        #[symbol = "_ui_opaque_unknown_key_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
