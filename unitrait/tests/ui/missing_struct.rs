unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_missing_struct"]
        fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
