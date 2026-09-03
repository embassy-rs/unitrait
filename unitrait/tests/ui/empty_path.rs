unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_empty_path"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = );
}

fn main() {}
