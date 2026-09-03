//! An opaque type without a `Drop` bound has no drop glue, and the implementation's
//! associated type is required to have none either.

unitrait::unitrait! {
    /// A test trait handing out move-only handles that need no cleanup.
    pub trait RegistryDriver {
        /// An opaque handle. It has no `Drop` bound, so dropping one does nothing and the
        /// implementation's type must need no dropping. It isn't `Copy` either, so it's
        /// still move-only.
        #[opaque(size = 8, align = 8)]
        pub type Handle;

        #[symbol = "_unitrait_test_registry_open"]
        fn registry_open(id: u32) -> Self::Handle;

        #[symbol = "_unitrait_test_registry_id"]
        fn registry_id(h: &Self::Handle) -> u32;

        #[symbol = "_unitrait_test_registry_close"]
        fn registry_close(h: Self::Handle) -> u32;
    }

    pub struct Registry;

    /// Set the global registry implementation.
    macro test_registry_impl(path = $crate);
}

struct MyRegistry;

/// Not `Copy`, but with no drop glue.
struct MyHandle {
    id: u32,
}

impl RegistryDriver for MyRegistry {
    type Handle = MyHandle;

    fn registry_open(id: u32) -> MyHandle {
        MyHandle { id }
    }

    fn registry_id(h: &MyHandle) -> u32 {
        h.id
    }

    fn registry_close(h: MyHandle) -> u32 {
        h.id
    }
}

test_registry_impl!(MyRegistry);

#[test]
fn test_no_drop_glue() {
    assert!(!core::mem::needs_drop::<RegistryHandle>());
}

// `drop` on a type with no drop glue is exactly what's being tested here.
#[allow(clippy::drop_non_drop)]
#[test]
fn test_round_trip() {
    let h = Registry::registry_open(7);
    assert_eq!(Registry::registry_id(&h), 7);
    assert_eq!(Registry::registry_close(h), 7);

    // Dropping one without handing it back is a no-op, not a leak.
    let h = Registry::registry_open(9);
    assert_eq!(Registry::registry_id(&h), 9);
    drop(h);
}
