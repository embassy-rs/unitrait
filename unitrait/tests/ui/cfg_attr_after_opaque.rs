unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[cfg_attr(all(), opaque(size = 16, align = 8))]
        pub type Context;

        #[symbol = "_ui_cfg_attr_after_opaque_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
