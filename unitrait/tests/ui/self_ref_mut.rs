unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_self_ref_mut"]
        fn foo(&mut self) -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
