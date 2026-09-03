unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[clone_symbol = "_ui_copy_with_clone_bound_clone"]
        pub type Context: Copy + Clone;

        #[symbol = "_ui_copy_with_clone_bound_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
