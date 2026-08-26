unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_dup_assoc_drop_a"]
        pub type Context;

        #[opaque(size = 16, align = 8)]
        #[symbol = "_ui_dup_assoc_drop_b"]
        pub type Context;

        #[symbol = "_ui_dup_assoc_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
