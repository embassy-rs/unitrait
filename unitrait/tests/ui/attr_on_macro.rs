unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_attr_on_macro"]
        pub fn foo() -> u32;
    }

    #[macro_export]
    macro foo_impl(path = $crate);
}

fn main() {}
