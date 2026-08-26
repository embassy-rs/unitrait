unitrait::unitrait! {
    pub trait Foo {
        #[inline]
        #[symbol = "_ui_attr_on_method"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
