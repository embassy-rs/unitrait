unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_nested_ref_array_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_nested_ref_array"]
        fn poke(ctx: &mut [Self::Context; 4]);
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
