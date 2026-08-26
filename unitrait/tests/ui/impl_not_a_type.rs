unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_impl_not_a_type"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

foo_impl!(struct);

fn main() {}
