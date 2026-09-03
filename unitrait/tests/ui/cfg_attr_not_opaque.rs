unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[cfg_attr(all(), drop_symbol = "_ui_cfg_attr_not_opaque_drop")]
        pub type Context: Drop;

        #[symbol = "_ui_cfg_attr_not_opaque_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
