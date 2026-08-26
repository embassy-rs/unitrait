unitrait::unitrait! {
    pub trait Foo {
        #[symbol = 42]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
