unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_method_with_visibility"]
        pub fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
