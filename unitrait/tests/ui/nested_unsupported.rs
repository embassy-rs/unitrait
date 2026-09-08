unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_nested_unsupported_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_nested_unsupported"]
        fn poke(ctx: Box<Self::Context>);
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
