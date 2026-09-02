unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_bound_path_drop"]
        pub type Context: core::marker::Send;

        #[symbol = "_ui_bound_path_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
