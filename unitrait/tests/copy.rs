//! `Copy` opaque types have no drop glue and can be duplicated freely.

unitrait::unitrait! {
    /// A test trait handing out copyable opaque tokens.
    pub trait TokensDriver {
        /// An opaque token. `Copy`, so it can't have a `Drop` bound and is never dropped.
        #[opaque(size = 8, align = 4)]
        pub type Token: Copy;

        /// A `Copy` token that is also `Send` and `Sync`.
        #[opaque(size = 4, align = 4)]
        pub type Small: Copy + Send + Sync;

        #[symbol = "_unitrait_test_token_new"]
        fn token_new(v: u32) -> Self::Token;

        #[symbol = "_unitrait_test_token_get"]
        fn token_get(t: Self::Token) -> u32;

        #[symbol = "_unitrait_test_token_peek"]
        fn token_peek(t: &Self::Token) -> u32;

        #[symbol = "_unitrait_test_small_new"]
        fn small_new(v: u32) -> Self::Small;

        #[symbol = "_unitrait_test_small_get"]
        fn small_get(t: Self::Small) -> u32;
    }

    pub struct Tokens;

    /// Set the global token implementation.
    macro test_tokens_impl(path = $crate);
}

struct MyImpl;

#[derive(Clone, Copy)]
struct MyToken {
    lo: u32,
    hi: u32,
}

impl TokensDriver for MyImpl {
    type Token = MyToken;
    type Small = u32;

    fn token_new(v: u32) -> MyToken {
        MyToken { lo: v, hi: !v }
    }

    fn token_get(t: MyToken) -> u32 {
        // Both halves must have survived the round trip through the opaque type.
        assert_eq!(t.hi, !t.lo);
        t.lo
    }

    fn token_peek(t: &MyToken) -> u32 {
        t.lo
    }

    fn small_new(v: u32) -> u32 {
        v
    }

    fn small_get(t: u32) -> u32 {
        t
    }
}

test_tokens_impl!(MyImpl);

fn assert_copy<T: ::core::marker::Copy>() {}
fn assert_send<T: ::core::marker::Send>() {}
fn assert_sync<T: ::core::marker::Sync>() {}

#[test]
fn test_copy_opaque_is_copy() {
    assert_copy::<TokensToken>();
    assert_copy::<TokensSmall>();
    assert_send::<TokensSmall>();
    assert_sync::<TokensSmall>();
}

#[test]
fn test_copy_opaque_has_no_drop_glue() {
    assert!(!core::mem::needs_drop::<TokensToken>());
    assert!(!core::mem::needs_drop::<TokensSmall>());
}

#[test]
fn test_copy_opaque_round_trips() {
    let t = Tokens::token_new(0x1234);
    // `t` is `Copy`: passing it by value doesn't move it out.
    assert_eq!(Tokens::token_get(t), 0x1234);
    assert_eq!(Tokens::token_get(t), 0x1234);
    assert_eq!(Tokens::token_peek(&t), 0x1234);

    let copy = t;
    assert_eq!(Tokens::token_get(copy), Tokens::token_get(t));

    #[allow(clippy::clone_on_copy)]
    let cloned = t.clone();
    assert_eq!(Tokens::token_get(cloned), 0x1234);
}

#[test]
fn test_copy_opaque_layout() {
    assert_eq!(core::mem::size_of::<TokensToken>(), 8);
    assert_eq!(core::mem::align_of::<TokensToken>(), 4);
    assert_eq!(Tokens::small_get(Tokens::small_new(9)), 9);
}
