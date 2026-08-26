unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_nested_self_drop"]
        pub type Context;

        #[symbol = "_ui_nested_self_new"]
        pub fn new() -> Option<Self::Context>;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
