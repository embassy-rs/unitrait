unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_nested_qualified_option_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_nested_qualified_option"]
        fn get() -> core::option::Option<Self::Context>;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
