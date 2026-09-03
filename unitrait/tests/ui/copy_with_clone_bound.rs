unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[clone_symbol = "_ui_copy_with_clone_bound_clone"]
        pub type Context: Copy + Clone;

        #[symbol = "_ui_copy_with_clone_bound_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
