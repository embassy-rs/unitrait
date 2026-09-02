//! Auto traits of opaque types follow the bounds declared on the associated type.

use std::sync::atomic::{AtomicU32, Ordering};

unitrait::unitrait! {
    /// A test trait declaring opaque types with various marker bounds.
    pub trait Marked {
        /// No bounds: the opaque type implements no auto trait.
        #[opaque(size = 8, align = 8)]
        #[symbol = "_unitrait_test_marked_bare_drop"]
        pub type Bare;

        /// Every supported auto trait.
        #[opaque(size = 16, align = 8)]
        #[symbol = "_unitrait_test_marked_all_drop"]
        pub type All: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe;

        /// Only `Send`, so it can be moved to another thread but not shared.
        #[opaque(size = 8, align = 8)]
        #[symbol = "_unitrait_test_marked_owned_drop"]
        pub type Owned: Send;

        #[symbol = "_unitrait_test_marked_bare_new"]
        pub fn bare_new() -> Self::Bare;

        #[symbol = "_unitrait_test_marked_all_new"]
        pub fn all_new(v: u32) -> Self::All;

        #[symbol = "_unitrait_test_marked_all_get"]
        pub fn all_get(ctx: &Self::All) -> u32;

        #[symbol = "_unitrait_test_marked_owned_new"]
        pub fn owned_new(v: u32) -> Self::Owned;

        #[symbol = "_unitrait_test_marked_owned_get"]
        pub fn owned_get(ctx: &Self::Owned) -> u32;
    }

    /// Set the global implementation.
    macro test_marked_impl(path = $crate);
}

static DROPS: AtomicU32 = AtomicU32::new(0);

struct MyImpl;

/// Not `Send` nor `Sync`, matching the bound-less `Bare`.
struct BareState(#[allow(dead_code)] std::rc::Rc<u32>);

struct Counted(u32);

impl Drop for Counted {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

impl Marked for MyImpl {
    type Bare = BareState;
    type All = u64;
    // The only type with drop glue, so `DROPS` only counts `Owned` values.
    type Owned = Counted;

    fn bare_new() -> BareState {
        BareState(std::rc::Rc::new(0))
    }

    fn all_new(v: u32) -> u64 {
        v as u64
    }

    fn all_get(ctx: &u64) -> u32 {
        *ctx as u32
    }

    fn owned_new(v: u32) -> Counted {
        Counted(v)
    }

    fn owned_get(ctx: &Counted) -> u32 {
        ctx.0
    }
}

test_marked_impl!(MyImpl);

fn assert_send<T: ::core::marker::Send>() {}
fn assert_sync<T: ::core::marker::Sync>() {}
fn assert_unpin<T: ::core::marker::Unpin>() {}
fn assert_unwind_safe<T: ::core::panic::UnwindSafe>() {}
fn assert_ref_unwind_safe<T: ::core::panic::RefUnwindSafe>() {}

#[test]
fn test_declared_bounds_are_implemented() {
    assert_send::<MarkedAll>();
    assert_sync::<MarkedAll>();
    assert_unpin::<MarkedAll>();
    assert_unwind_safe::<MarkedAll>();
    assert_ref_unwind_safe::<MarkedAll>();
    assert_send::<MarkedOwned>();
}

#[test]
fn test_bounds_do_not_change_layout() {
    assert_eq!(core::mem::size_of::<MarkedBare>(), 8);
    assert_eq!(core::mem::align_of::<MarkedBare>(), 8);
    assert_eq!(core::mem::size_of::<MarkedAll>(), 16);
    assert_eq!(core::mem::align_of::<MarkedAll>(), 8);
}

#[test]
fn test_send_opaque_crosses_threads() {
    let before = DROPS.load(Ordering::Relaxed);
    let ctx = owned_new(21);
    let value = std::thread::spawn(move || {
        let v = owned_get(&ctx);
        drop(ctx);
        v
    })
    .join()
    .unwrap();
    assert_eq!(value, 21);
    assert_eq!(DROPS.load(Ordering::Relaxed), before + 1);
}

#[test]
fn test_bare_opaque_still_works_locally() {
    let ctx = bare_new();
    drop(ctx);
    let ctx = all_new(5);
    assert_eq!(all_get(&ctx), 5);
}
