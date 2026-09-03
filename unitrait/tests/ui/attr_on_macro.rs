unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_attr_on_macro"]
        fn foo() -> u32;
    }

    pub struct Foo;

    #[macro_export]
    macro foo_impl(path = $crate);
}

fn main() {}
