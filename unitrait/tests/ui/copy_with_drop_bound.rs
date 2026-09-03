unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        pub type Context: Copy + Drop;

        #[symbol = "_ui_copy_with_drop_bound_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
