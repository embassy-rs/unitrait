unitrait::unitrait! {
    #[derive(Debug)]
    pub trait FooDriver {
        #[symbol = "_ui_attr_on_trait"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
