unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_bound_duplicate_drop"]
        pub type Context: Send + Sync + Send;

        #[symbol = "_ui_bound_duplicate_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
