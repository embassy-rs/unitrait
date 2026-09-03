unitrait::unitrait! {
    pub trait Foo {
        #[cfg_attr(all(), opaque(size = 8, align = 4), opaque(size = 16, align = 8))]
        #[opaque(size = 8, align = 4)]
        pub type Context;

        #[symbol = "_ui_cfg_attr_duplicate_opaque_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
