unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_unknown_assoc_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
