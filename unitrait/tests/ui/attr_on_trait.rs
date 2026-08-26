unitrait::unitrait! {
    #[derive(Debug)]
    pub trait Foo {
        #[symbol = "_ui_attr_on_trait"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
