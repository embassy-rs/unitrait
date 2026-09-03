unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_opaque_not_auto_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_opaque_not_auto_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl FooDriver for MyImpl {
    type Context = u32;

    fn new() -> u32 {
        0
    }
}

foo_impl!(MyImpl);

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}
fn assert_unpin<T: Unpin>() {}
fn assert_unwind_safe<T: std::panic::UnwindSafe>() {}
fn assert_ref_unwind_safe<T: std::panic::RefUnwindSafe>() {}

fn main() {
    // The associated type declares no bounds, so the opaque type implements no auto trait,
    // whatever the implementation's associated type happens to be.
    assert_send::<FooContext>();
    assert_sync::<FooContext>();
    assert_unpin::<FooContext>();
    assert_unwind_safe::<FooContext>();
    assert_ref_unwind_safe::<FooContext>();
}
