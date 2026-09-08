//! `Option` and `Result` shadowed by unrelated local types.
//!
//! Around an opaque associated type, `Option` and `Result` are matched by their bare names
//! and always mean the `core` types, so shadowing must not change what the generated code
//! means. Away from opaque types, a signature is passed through verbatim, so there the
//! shadowing types are used as written.

#![allow(dead_code)]

/// Shadows `core::option::Option`. A struct, so `Some`/`None` can't be its variants.
pub struct Option<T>(pub T);
/// Shadows `core::result::Result`, with the parameters swapped for good measure.
pub struct Result<E, T>(pub E, pub T);
pub struct Pin<T>(pub T);
pub mod core {}
pub mod option {}
pub mod result {}

unitrait::unitrait! {
    /// A test trait defined with `Option` and `Result` shadowed.
    pub trait ShadowedDriver {
        #[opaque(size = 16, align = 8)]
        #[drop_symbol = "_unitrait_test_nested_shadow_drop"]
        pub type Context: Drop;

        /// `core::option::Option`, whatever `Option` is in scope.
        #[symbol = "_unitrait_test_nested_shadow_new"]
        fn shadow_new(v: u32) -> Option<Self::Context>;

        /// Both: the outer `Option` is around an opaque type, so it is `core`'s; the
        /// inner one isn't, so it's the local struct.
        #[symbol = "_unitrait_test_nested_shadow_get"]
        fn shadow_get(ctx: Option<&Self::Context>, plain: Option<u32>) -> Result<u32, Self::Context>;

        /// No opaque type: the local `Option` and `Result`, passed through verbatim.
        #[symbol = "_unitrait_test_nested_shadow_plain"]
        fn shadow_plain(v: Option<u32>) -> Result<u32, u32>;
    }

    pub struct Shadowed;

    /// Set the global implementation.
    macro test_nested_shadow_impl(path = $crate);
}

struct MyImpl;

struct MyState(u32);

impl ShadowedDriver for MyImpl {
    type Context = MyState;

    // `Option` and `Result` are shadowed here too, so the impl must spell out the absolute
    // paths, exactly like the trait declaration does after `unitrait!` rewrote it.
    fn shadow_new(v: u32) -> ::core::option::Option<MyState> {
        ::core::option::Option::Some(MyState(v))
    }

    fn shadow_get(
        ctx: ::core::option::Option<&MyState>,
        plain: Option<u32>,
    ) -> ::core::result::Result<u32, MyState> {
        match ctx {
            ::core::option::Option::Some(ctx) => ::core::result::Result::Ok(ctx.0 + plain.0),
            ::core::option::Option::None => ::core::result::Result::Err(MyState(plain.0)),
        }
    }

    fn shadow_plain(v: Option<u32>) -> Result<u32, u32> {
        Result(v.0, v.0 + 1)
    }
}

test_nested_shadow_impl!(MyImpl);

#[test]
fn test_shadowed_wrappers_mean_the_core_types_around_opaque_types() {
    let ctx: ::core::option::Option<ShadowedContext> = Shadowed::shadow_new(5);
    let ctx = ctx.unwrap();
    let r: ::core::result::Result<u32, ShadowedContext> =
        Shadowed::shadow_get(::core::option::Option::Some(&ctx), Option(10));
    let ::core::result::Result::Ok(15) = r else {
        panic!()
    };
    let r = Shadowed::shadow_get(::core::option::Option::None, Option(7));
    let ::core::result::Result::Err(_ctx) = r else {
        panic!()
    };
}

#[test]
fn test_shadowed_wrappers_are_passed_through_without_opaque_types() {
    let r: Result<u32, u32> = Shadowed::shadow_plain(Option(3));
    assert_eq!((r.0, r.1), (3, 4));
}
