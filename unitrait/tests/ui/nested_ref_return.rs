unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_nested_ref_return_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_nested_ref_return"]
        fn get() -> Option<&Self::Context>;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
