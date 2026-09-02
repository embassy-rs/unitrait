unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_bound_maybe_sized_drop"]
        pub type Context: ?Sized;

        #[symbol = "_ui_bound_maybe_sized_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
