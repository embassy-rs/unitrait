//! Everything the generated code names is shadowed here by unrelated local items.
//!
//! Bounds are matched by their bare name and always mean the `core` trait, and `Pin` is
//! always `core::pin::Pin`, so shadowing must not change what the generated bounds, the
//! `impl`s on the opaque structs, or the shims mean.

#![allow(dead_code)]

pub trait Send {}
pub trait Sync {}
pub trait Unpin {}
pub trait UnwindSafe {}
pub trait RefUnwindSafe {}
pub trait Copy {}
pub trait Clone {}
pub trait Drop {}
pub trait Sized {}
pub struct Pin<T>(pub T);
pub struct PhantomData<T>(pub T);
pub struct PhantomPinned;
pub struct MaybeUninit<T>(pub T);
pub struct UnsafeCell<T>(pub T);
pub struct ManuallyDrop<T>(pub T);
pub mod marker {}
pub mod mem {}
pub mod ptr {}
pub mod pin {}
pub mod panic {}
pub mod ops {}
pub mod cell {}
pub mod clone {}
pub mod core {}

unitrait::unitrait! {
    /// A test trait defined with every relevant name shadowed.
    pub trait Shadowed {
        /// An opaque type with every marker bound, in a scope where they're all shadowed.
        #[opaque(size = 16, align = 8)]
        #[drop_symbol = "_unitrait_test_shadow_ctx_drop"]
        pub type Context: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + Drop;

        /// A `Copy` opaque type, in a scope where `Copy` and `Clone` are shadowed.
        #[opaque(size = 4, align = 4)]
        pub type Token: Copy;

        #[symbol = "_unitrait_test_shadow_new"]
        pub fn shadow_new(v: u32) -> Self::Context;

        /// Uses the shadowed `Pin`: it must still mean `core::pin::Pin`.
        #[symbol = "_unitrait_test_shadow_bump"]
        pub fn shadow_bump(ctx: Pin<&mut Self::Context>) -> u32;

        #[symbol = "_unitrait_test_shadow_get"]
        pub fn shadow_get(ctx: Pin<&Self::Context>) -> u32;

        #[symbol = "_unitrait_test_shadow_token"]
        pub fn shadow_token(v: u32) -> Self::Token;

        #[symbol = "_unitrait_test_shadow_token_get"]
        pub fn shadow_token_get(t: Self::Token) -> u32;
    }

    /// Set the global implementation.
    macro test_shadow_impl(path = $crate);
}

struct MyImpl;

struct MyState(u32);

impl Shadowed for MyImpl {
    type Context = MyState;
    type Token = u32;

    fn shadow_new(v: u32) -> MyState {
        MyState(v)
    }

    // `Pin` is shadowed here too, so the impl must spell out the absolute path, exactly
    // like the trait declaration does after `unitrait!` rewrote it.
    fn shadow_bump(ctx: ::core::pin::Pin<&mut MyState>) -> u32 {
        // SAFETY: nothing is moved out of the state.
        let ctx = unsafe { ctx.get_unchecked_mut() };
        ctx.0 += 1;
        ctx.0
    }

    fn shadow_get(ctx: ::core::pin::Pin<&MyState>) -> u32 {
        ctx.0
    }

    fn shadow_token(v: u32) -> u32 {
        v
    }

    fn shadow_token_get(t: u32) -> u32 {
        t
    }
}

test_shadow_impl!(MyImpl);

fn assert_send<T: ::core::marker::Send>() {}
fn assert_sync<T: ::core::marker::Sync>() {}
fn assert_unpin<T: ::core::marker::Unpin>() {}
fn assert_unwind_safe<T: ::core::panic::UnwindSafe>() {}
fn assert_ref_unwind_safe<T: ::core::panic::RefUnwindSafe>() {}
fn assert_copy<T: ::core::marker::Copy>() {}

#[test]
fn test_shadowed_bounds_mean_the_core_traits() {
    // Not the local `Send`/`Sync`/... traits declared above, which nothing implements.
    assert_send::<ShadowedContext>();
    assert_sync::<ShadowedContext>();
    assert_unpin::<ShadowedContext>();
    assert_unwind_safe::<ShadowedContext>();
    assert_ref_unwind_safe::<ShadowedContext>();
    assert_copy::<ShadowedToken>();
}

#[test]
fn test_shadowed_names_still_dispatch() {
    let mut ctx = ::core::pin::pin!(shadow_new(1));
    assert_eq!(shadow_bump(ctx.as_mut()), 2);
    assert_eq!(shadow_get(ctx.as_ref()), 2);
    assert_eq!(shadow_token_get(shadow_token(7)), 7);
}

#[test]
fn test_shadowed_opaque_layout() {
    assert_eq!(::core::mem::size_of::<ShadowedContext>(), 16);
    assert_eq!(::core::mem::align_of::<ShadowedContext>(), 8);
}
