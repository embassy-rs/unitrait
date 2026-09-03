unitrait::unitrait! {
    pub trait Foo {
        #[cfg_attr(any(), opaque(size = 8, align = 4))]
        #[cfg_attr(target_os = "none", opaque(size = 16, align = 8))]
        pub type Context;

        #[symbol = "_ui_cfg_attr_no_match_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
