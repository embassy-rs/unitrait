unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_nested_pin_by_value_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_nested_pin_by_value"]
        fn poke(ctx: [Pin<Self::Context>; 2]);
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
