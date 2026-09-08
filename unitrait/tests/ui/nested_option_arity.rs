unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_nested_option_arity_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_nested_option_arity"]
        fn poke(ctx: Option<Self::Context, u32>);
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
