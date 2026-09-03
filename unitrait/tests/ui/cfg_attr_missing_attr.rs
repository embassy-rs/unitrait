unitrait::unitrait! {
    pub trait Foo {
        #[cfg_attr(all())]
        #[opaque(size = 8, align = 4)]
        pub type Context;

        #[symbol = "_ui_cfg_attr_missing_attr_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
