unitrait::unitrait! {
    pub trait Foo {
        #[cfg_attr(all(), opaque(size = 8, align = 4))]
        #[symbol = "_ui_cfg_attr_on_method"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
