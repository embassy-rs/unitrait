unitrait::unitrait! {
    pub trait FooDriver {
        #[cfg_attr(all(), opaque(size = 8, align = 4))]
        #[symbol = "_ui_cfg_attr_on_method"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
