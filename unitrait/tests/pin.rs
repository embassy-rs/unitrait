//! `Pin<&Self::Name>` and `Pin<&mut Self::Name>` parameters.

use core::pin::{Pin, pin};
use std::sync::atomic::{AtomicU32, Ordering};

unitrait::unitrait! {
    /// A test trait whose state must not move once started.
    pub trait Anchored {
        /// Opaque storage for the implementation's state.
        #[opaque(size = 32, align = 8)]
        #[symbol = "_unitrait_test_anchored_drop"]
        pub type State;

        #[symbol = "_unitrait_test_anchored_new"]
        pub fn anchored_new(v: u32) -> Self::State;

        /// Records the address the state was pinned at.
        #[symbol = "_unitrait_test_anchored_start"]
        pub fn anchored_start(state: Pin<&mut Self::State>);

        /// Bumps the value, checking the state hasn't moved since `anchored_start`.
        #[symbol = "_unitrait_test_anchored_bump"]
        pub fn anchored_bump(state: Pin<&mut Self::State>) -> u32;

        /// Reads the value through a pinned shared reference.
        #[symbol = "_unitrait_test_anchored_get"]
        pub fn anchored_get(state: Pin<&Self::State>) -> u32;
    }

    /// Set the global implementation.
    macro test_anchored_impl(path = $crate);
}

static DROPS: AtomicU32 = AtomicU32::new(0);

struct MyImpl;

struct MyState {
    value: u32,
    /// Set by `anchored_start` to the address the state was pinned at.
    anchor: *const MyState,
}

impl Drop for MyState {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

impl MyState {
    fn check_anchor(&self) {
        assert_eq!(
            self.anchor, self as *const MyState,
            "the state moved while pinned"
        );
    }
}

impl Anchored for MyImpl {
    type State = MyState;

    fn anchored_new(v: u32) -> MyState {
        MyState {
            value: v,
            anchor: core::ptr::null(),
        }
    }

    fn anchored_start(state: Pin<&mut MyState>) {
        // SAFETY: the field is never moved out of.
        let state = unsafe { state.get_unchecked_mut() };
        state.anchor = state as *const MyState;
    }

    fn anchored_bump(state: Pin<&mut MyState>) -> u32 {
        // SAFETY: the field is never moved out of.
        let state = unsafe { state.get_unchecked_mut() };
        state.check_anchor();
        state.value += 1;
        state.value
    }

    fn anchored_get(state: Pin<&MyState>) -> u32 {
        state.check_anchor();
        state.value
    }
}

test_anchored_impl!(MyImpl);

fn assert_unpin<T: ::core::marker::Unpin>() {}

#[test]
fn test_pinned_state_keeps_its_address() {
    let before = DROPS.load(Ordering::Relaxed);
    {
        let mut state = pin!(anchored_new(10));
        anchored_start(state.as_mut());
        assert_eq!(anchored_bump(state.as_mut()), 11);
        assert_eq!(anchored_bump(state.as_mut()), 12);
        assert_eq!(anchored_get(state.as_ref()), 12);
    }
    assert_eq!(DROPS.load(Ordering::Relaxed), before + 1);
}

#[test]
fn test_pinning_on_the_heap() {
    let mut state = Box::pin(anchored_new(0));
    anchored_start(state.as_mut());
    assert_eq!(anchored_bump(state.as_mut()), 1);
    assert_eq!(anchored_get(state.as_ref()), 1);
}

#[test]
fn test_opaque_without_unpin_bound_is_not_unpin() {
    // The opaque type is `!Unpin`, so a `Pin<&mut AnchoredState>` can only be built by
    // actually pinning the value: that is what makes the `Pin` parameters sound.
    assert_unpin::<u32>();
    // `assert_unpin::<AnchoredState>()` would not compile; see tests/ui/opaque_not_auto.rs.
}
