unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_bad_path_kw"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(paths = $crate);
}

fn main() {}
