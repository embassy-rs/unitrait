unitrait::unitrait! {
    pub trait FooDriver {
        #[inline]
        #[symbol = "_ui_attr_on_method"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
