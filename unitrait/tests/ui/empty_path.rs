unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_empty_path"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = );
}

fn main() {}
