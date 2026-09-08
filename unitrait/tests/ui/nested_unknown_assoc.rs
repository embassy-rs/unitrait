unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_nested_unknown_assoc_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_nested_unknown_assoc"]
        fn get() -> Option<Self::Nope>;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
