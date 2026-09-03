unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        pub type Context: Copy + Drop;

        #[symbol = "_ui_copy_with_drop_bound_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
