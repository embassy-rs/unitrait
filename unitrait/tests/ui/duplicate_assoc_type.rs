unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_dup_assoc_drop_a"]
        pub type Context: Drop;

        #[opaque(size = 16, align = 8)]
        #[drop_symbol = "_ui_dup_assoc_drop_b"]
        pub type Context: Drop;

        #[symbol = "_ui_dup_assoc_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
