unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_ref_lifetime_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_ref_lifetime_peek"]
        fn peek(ctx: &'static Self::Context) -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
