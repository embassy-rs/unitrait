unitrait::unitrait! {
    #[symbol_prefix = 42]
    pub trait Foo {
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
