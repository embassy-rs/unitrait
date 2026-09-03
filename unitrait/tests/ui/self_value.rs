unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_self_value"]
        fn foo(self) -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
