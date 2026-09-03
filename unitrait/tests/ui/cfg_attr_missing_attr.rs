unitrait::unitrait! {
    pub trait FooDriver {
        #[cfg_attr(all())]
        #[opaque(size = 8, align = 4)]
        pub type Context;

        #[symbol = "_ui_cfg_attr_missing_attr_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
