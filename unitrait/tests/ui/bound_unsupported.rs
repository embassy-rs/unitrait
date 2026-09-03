unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        pub type Context: Default;

        #[symbol = "_ui_bound_unsupported_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
