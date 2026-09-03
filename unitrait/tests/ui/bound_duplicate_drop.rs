unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_bound_duplicate_drop_drop"]
        pub type Context: Drop + Drop;

        #[symbol = "_ui_bound_duplicate_drop_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
