unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_bad_path_kw"]
        pub fn foo() -> u32;
    }

    macro foo_impl(paths = $crate);
}

fn main() {}
