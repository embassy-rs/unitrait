unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_struct_named_like_trait"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
