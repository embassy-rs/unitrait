//! `#[symbol_prefix]` derives every symbol name, with per-item attributes as overrides.

unitrait::unitrait! {
    /// A test trait deriving its symbol names from a prefix.
    #[symbol_prefix = "_unitrait_test_prefixed"]
    pub trait PrefixedDriver {
        /// Symbols `_unitrait_test_prefixed_Context_drop` and `..._Context_clone`.
        #[opaque(size = 32, align = 8)]
        pub type Context: Clone + Drop;

        /// A `Copy` opaque type needs no symbols of its own.
        #[opaque(size = 4, align = 4)]
        pub type Token: Copy;

        /// Symbol `_unitrait_test_prefixed_ctx_new`.
        #[symbol = "_unitrait_test_prefixed_ctx_new"]
        fn ctx_new(v: u32) -> Self::Context;

        /// Derived symbol.
        fn ctx_get(ctx: &Self::Context) -> u32;

        /// An explicit override, not `_unitrait_test_prefixed_ctx_token`.
        #[symbol = "_unitrait_test_prefixed_override"]
        fn ctx_token(ctx: &Self::Context) -> Self::Token;

        /// Derived symbol on an `unsafe` method.
        unsafe fn token_get(t: Self::Token) -> u32;
    }

    pub struct Prefixed;

    /// Set the global prefixed implementation.
    macro test_prefixed_impl(path = $crate);
}

struct MyImpl;

#[derive(Clone)]
struct MyContext {
    v: u32,
}

impl Drop for MyContext {
    fn drop(&mut self) {}
}

impl PrefixedDriver for MyImpl {
    type Context = MyContext;
    type Token = u32;

    fn ctx_new(v: u32) -> MyContext {
        MyContext { v }
    }

    fn ctx_get(ctx: &MyContext) -> u32 {
        ctx.v
    }

    fn ctx_token(ctx: &MyContext) -> u32 {
        ctx.v ^ 0xffff
    }

    unsafe fn token_get(t: u32) -> u32 {
        t ^ 0xffff
    }
}

test_prefixed_impl!(MyImpl);

// The derived names are part of the ABI, so spell them out: linking fails if the emitted
// names don't match.
unsafe extern "Rust" {
    #[link_name = "_unitrait_test_prefixed_ctx_get"]
    safe fn raw_ctx_get(ctx: &PrefixedContext) -> u32;

    // Declared `unsafe`, matching the dispatch method generated for the `unsafe` method.
    #[link_name = "_unitrait_test_prefixed_token_get"]
    fn raw_token_get(t: PrefixedToken) -> u32;

    #[link_name = "_unitrait_test_prefixed_Context_drop"]
    safe fn raw_ctx_drop(ctx: &mut PrefixedContext);

    #[link_name = "_unitrait_test_prefixed_Context_clone"]
    safe fn raw_ctx_clone(ctx: &PrefixedContext) -> PrefixedContext;
}

#[test]
fn test_derived_symbols_dispatch() {
    let ctx = Prefixed::ctx_new(7);
    assert_eq!(Prefixed::ctx_get(&ctx), 7);
    assert_eq!(raw_ctx_get(&ctx), 7);

    let t = Prefixed::ctx_token(&ctx);
    assert_eq!(unsafe { Prefixed::token_get(t) }, 7);
    assert_eq!(unsafe { raw_token_get(t) }, 7);
}

#[test]
fn test_derived_drop_and_clone_symbols() {
    let ctx = Prefixed::ctx_new(3);
    let mut cloned = raw_ctx_clone(&ctx);
    assert_eq!(Prefixed::ctx_get(&cloned), 3);

    // `clone` on the opaque type reaches the same symbol.
    assert_eq!(Prefixed::ctx_get(&ctx.clone()), 3);

    // Dropping in place is what `Drop for PrefixedContext` does; forget the husk so it
    // doesn't happen twice.
    raw_ctx_drop(&mut cloned);
    core::mem::forget(cloned);
}
