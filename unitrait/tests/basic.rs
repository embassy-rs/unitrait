use std::sync::atomic::{AtomicU32, Ordering};

pub struct Item(pub u64);

unitrait::unitrait! {
    /// A test driver.
    pub trait Driver {
        /// Returns the current level.
        #[symbol = "_unitrait_test_level"]
        fn level() -> u32;

        /// Sets the current level.
        #[symbol = "_unitrait_test_set_level"]
        fn set_level(level: u32, doubled: bool);

        /// No args, no return value.
        #[symbol = "_unitrait_test_poke"]
        fn poke();

        /// Returns a static item.
        ///
        /// # Safety
        ///
        /// May be called at most once.
        #[symbol = "_unitrait_test_item"]
        unsafe fn item(marker: &u8) -> &'static mut Item;
    }

    /// The global test driver.
    pub struct Frob;

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
fn test_dispatch_type_dispatches_to_impl() {
    assert_eq!(Frob::level(), 7);
    Frob::set_level(10, true);
    assert_eq!(Frob::level(), 20);
    Frob::poke();
    assert_eq!(Frob::level(), 21);
    let item = unsafe { Frob::item(&0) };
    assert_eq!(item.0, 42);
}

fn via_trait<T: Driver>(level: u32) -> u32 {
    T::set_level(level, false);
    T::level()
}

#[test]
fn test_dispatch_type_implements_trait() {
    // The dispatch type is an implementation of the trait, usable in generic code.
    assert_eq!(via_trait::<Frob>(5), 5);
    assert_eq!(via_trait::<MyDriver>(6), 6);
}
