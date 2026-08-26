use std::sync::atomic::{AtomicU32, Ordering};

pub struct Item(pub u64);

unitrait::unitrait! {
    /// A test driver.
    pub trait Driver {
        /// Returns the current level.
        #[symbol = "_unitrait_test_level"]
        pub fn level() -> u32;

        /// Sets the current level.
        #[symbol = "_unitrait_test_set_level"]
        pub(crate) fn set_level(level: u32, doubled: bool);

        /// No args, no return value. No visibility: the free function is private.
        #[symbol = "_unitrait_test_poke"]
        fn poke();

        /// Returns a static item.
        ///
        /// # Safety
        ///
        /// May be called at most once.
        #[symbol = "_unitrait_test_item"]
        pub unsafe fn item(marker: &u8) -> &'static mut Item;
    }

    /// Set the global test driver.
    macro test_driver_impl(path = $crate);
}

static LEVEL: AtomicU32 = AtomicU32::new(7);

struct MyDriver;

impl Driver for MyDriver {
    fn level() -> u32 {
        LEVEL.load(Ordering::Relaxed)
    }

    fn set_level(level: u32, doubled: bool) {
        LEVEL.store(if doubled { level * 2 } else { level }, Ordering::Relaxed);
    }

    fn poke() {
        LEVEL.fetch_add(1, Ordering::Relaxed);
    }

    unsafe fn item(_marker: &u8) -> &'static mut Item {
        static mut ITEM: Item = Item(42);
        unsafe { &mut *core::ptr::addr_of_mut!(ITEM) }
    }
}

test_driver_impl!(MyDriver);

#[test]
fn test_free_fns_dispatch_to_impl() {
    assert_eq!(level(), 7);
    set_level(10, true);
    assert_eq!(level(), 20);
    poke();
    assert_eq!(level(), 21);
    let item = unsafe { item(&0) };
    assert_eq!(item.0, 42);
}
