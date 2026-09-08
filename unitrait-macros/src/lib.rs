//! Implementation detail of the [`unitrait`](https://docs.rs/unitrait) crate.
//!
//! Use the `unitrait!` macro through the `unitrait` crate; do not depend on this crate
//! directly.

use proc_macro2::{Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Expr, GenericArgument, Ident, Lit, LitInt, LitStr, PathArguments, Token,
    TraitBoundModifier, Type, TypeParamBound, Visibility, braced, parenthesized,
};

/// Define a unitrait: a trait with a single global implementation, resolved at link time.
///
/// This macro takes a trait definition, the name of a "dispatch type" and the name of an
/// "implementation macro", and emits:
///
/// - The trait, as written.
/// - The dispatch type: a unit struct with one inherent method per trait method, with the
///   same name, signature and documentation, which calls the same method on the global
///   implementation. These methods are callable from anywhere in the crate tree, including
///   crates that can't see the implementation. The dispatch type also implements the trait,
///   forwarding to those methods, so it can be used wherever an implementation of the trait
///   is expected. Because the methods live on a type, several unitraits in one module may
///   use the same method names.
/// - The implementation macro, which registers a type as the global implementation of the
///   trait. It must be invoked exactly once across the whole crate tree; the program will
///   fail to link if it's invoked zero or multiple times.
///
/// # Syntax
///
/// ```
/// pub struct Item(pub u64);
///
/// unitrait::unitrait! {
///     /// A driver for the frobnicator.
///     pub trait Driver {
///         /// Returns the current frobnication level.
///         #[symbol = "_frob_level"]
///         fn level() -> u32;
///
///         /// Returns the item associated with `token`.
///         ///
///         /// # Safety
///         ///
///         /// `token` must have been obtained from [`Item::token`].
///         #[symbol = "_frob_item"]
///         unsafe fn item(token: u32) -> &'static mut Item;
///     }
///
///     /// The global frobnicator.
///     pub struct Frob;
///
///     /// Set the global frobnicator driver.
///     macro frob_driver_impl(path = $crate);
/// }
/// # struct MyDriver;
/// # impl Driver for MyDriver {
/// #     fn level() -> u32 { 42 }
/// #     unsafe fn item(_token: u32) -> &'static mut Item { unimplemented!() }
/// # }
/// # frob_driver_impl!(MyDriver);
/// # fn main() { assert_eq!(Frob::level(), 42); }
/// ```
///
/// Trait methods:
///
/// - must not have a `self` parameter. If the implementation needs state, it can store it
///   in `static`s, or in a [context](#opaque-associated-types) allocated by the caller.
/// - must each have a `#[symbol = "..."]` attribute specifying the extern symbol name used
///   for that method, unless the trait carries a [`#[symbol_prefix = "..."]`](#symbol-names)
///   attribute to derive one. Symbol names must be unique across the program: prefix them
///   with your crate's name and version. Changing a method's symbol name or signature is an
///   ABI-breaking change between the defining crate and implementor crates.
/// - may be `unsafe`. The corresponding dispatch method will be `unsafe` too.
/// - must not have a visibility: the dispatch methods are as visible as the dispatch type.
///
/// The `VIS struct NAME;` line names the dispatch type, which must not share the trait's
/// name. Its visibility is also that of its inherent methods: make it `pub(crate)` when only
/// the defining crate should call the unitrait, while the (more visible) trait stays
/// implementable by other crates. Attributes written on this line are applied to the struct
/// as they are, so it may for example `#[derive(...)]`.
///
/// The `macro NAME(path = PATH);` line sets the name of the generated implementation macro.
/// `PATH` must be the path of the module containing the `unitrait!` invocation, as seen from
/// other crates, starting with `$crate` — e.g. `$crate` if invoked at the crate root, or
/// `$crate::foo::bar` if invoked in the module `foo::bar`. It's used by the implementation
/// macro to name the trait and the dispatch type, so the module must be publicly reachable.
///
/// # Symbol names
///
/// Every extern symbol a unitrait uses can be written out one by one, as above, or derived
/// from a single `#[symbol_prefix = "..."]` attribute on the trait:
///
/// ```
/// unitrait::unitrait! {
///     /// A driver for the frobnicator.
///     #[symbol_prefix = "_frob_v1"]
///     pub trait Driver {
///         /// Uses the symbol `_frob_v1_Context_drop`.
///         #[opaque(size = 8, align = 8)]
///         pub type Context: Drop;
///
///         /// Uses the symbol `_frob_v1_level`.
///         fn level(ctx: &Self::Context) -> u32;
///
///         /// Overridden: uses `_frob_legacy_reset`, not `_frob_v1_reset`.
///         #[symbol = "_frob_legacy_reset"]
///         fn reset(ctx: &mut Self::Context);
///     }
///
///     /// The global frobnicator.
///     pub struct Frob;
///
///     /// Set the global frobnicator driver.
///     macro frob_driver_impl(path = $crate);
/// }
/// # struct MyDriver;
/// # struct MyCtx(u32);
/// # impl Drop for MyCtx { fn drop(&mut self) {} }
/// # impl Driver for MyDriver {
/// #     type Context = MyCtx;
/// #     fn level(ctx: &MyCtx) -> u32 { ctx.0 }
/// #     fn reset(ctx: &mut MyCtx) { ctx.0 = 0; }
/// # }
/// # frob_driver_impl!(MyDriver);
/// # fn main() {}
/// ```
///
/// With a prefix, the derived names are:
///
/// - `PREFIX_method_name` for each method.
/// - `PREFIX_TypeName_drop` and `PREFIX_TypeName_clone` for each [opaque associated
///   type](#opaque-associated-types) with a `Drop` or `Clone` bound. The associated type's
///   name is used as written, so it keeps its capitalization.
///
/// A `#[symbol = "..."]`, `#[drop_symbol = "..."]` or `#[clone_symbol = "..."]` attribute
/// written on an item overrides the derived name for that one symbol. Without a prefix on the
/// trait, every symbol must be written out explicitly. Derived names are as much a part of
/// the ABI as explicit ones: renaming the trait's prefix, a method, or an associated type is
/// an ABI-breaking change.
///
/// # Opaque associated types
///
/// Normally, unitrait methods can't mention associated types: callers dispatch through an
/// extern symbol and can't know what types the implementation chose, in particular their
/// size. A unitrait may however declare *opaque* associated types, which callers see as
/// opaque types of a fixed maximum size and alignment:
///
/// ```
/// unitrait::unitrait! {
///     /// A rolling checksum.
///     pub trait ChecksumDriver {
///         /// Opaque storage for the implementation's checksum state.
///         #[opaque(size = 16, align = 8)]
///         #[drop_symbol = "_cksum_context_drop"]
///         pub type Context: Drop;
///
///         /// Returns a fresh checksum state.
///         #[symbol = "_cksum_init"]
///         fn init() -> Self::Context;
///
///         /// Absorbs `data` into the checksum state.
///         #[symbol = "_cksum_update"]
///         fn update(ctx: &mut Self::Context, data: &[u8]);
///
///         /// Returns the checksum of the absorbed data.
///         #[symbol = "_cksum_finish"]
///         fn finish(ctx: &Self::Context) -> u32;
///     }
///
///     /// The global checksum.
///     pub struct Checksum;
///
///     /// Set the global checksum implementation.
///     macro checksum_impl(path = $crate);
/// }
/// # struct Fletcher;
/// # impl ChecksumDriver for Fletcher {
/// #     type Context = (u32, u32);
/// #     fn init() -> (u32, u32) { (0, 0) }
/// #     fn update(ctx: &mut (u32, u32), data: &[u8]) {
/// #         for &b in data { ctx.0 = (ctx.0 + b as u32) % 65535; ctx.1 = (ctx.1 + ctx.0) % 65535; }
/// #     }
/// #     fn finish(ctx: &(u32, u32)) -> u32 { ctx.1 << 16 | ctx.0 }
/// # }
/// # checksum_impl!(Fletcher);
/// # fn main() {
/// #     let mut ctx = Checksum::init();
/// #     Checksum::update(&mut ctx, b"hello");
/// #     assert_ne!(Checksum::finish(&ctx), 0);
/// # }
/// ```
///
/// Each opaque associated type declaration must be of the form `#[opaque(size = N, align = M)] [#[drop_symbol = "..."]] [#[clone_symbol = "..."]] [vis] type Name[: Bounds];`, where `N` and
/// `M` are integer literals, `M` is a power of two, and `Bounds` is described under [marker
/// bounds](#marker-bounds-on-opaque-types) and [dropping](#dropping-opaque-types). The
/// `#[opaque]` attribute may also be given [several times under
/// `cfg_attr`](#conditional-opaque-layouts). It emits:
///
/// - `type Name;` in the trait, with the declared bounds. The implementation sets it to a
///   type of its choosing, which must have size at most `N` rounded up to a multiple of `M`,
///   and alignment at most `M`; the implementation macro verifies both at compile time.
/// - The opaque struct, named by concatenating the dispatch type's name and the associated
///   type name (`ChecksumContext` above), laid out as `MaybeUninit<[u8; N]>` with `N`
///   rounded up to a multiple of `M`, and alignment `M`. Its visibility is the one written on
///   the `type` declaration (private if omitted); note the implementation macro and the
///   dispatch type's methods name it, so it must be visible wherever the trait is
///   implemented or the type is used. The dispatch type's impl of the trait sets
///   `type Name` to it.
///
/// An opaque struct value always holds an initialized value of the implementation's
/// (unknown to the caller) associated type: the only way to obtain one is through a method
/// that returns it. This is why methods taking opaque types are safe.
///
/// # Conditional opaque layouts
///
/// The size and alignment may depend on `cfg`s of the defining crate, by wrapping
/// `#[opaque]` in `#[cfg_attr(...)]`:
///
/// ```
/// unitrait::unitrait! {
///     /// A rolling checksum.
///     pub trait ChecksumDriver {
///         /// Opaque storage for the implementation's checksum state.
///         #[cfg_attr(feature = "wide-checksum", opaque(size = 64, align = 16))]
///         #[cfg_attr(target_pointer_width = "16", opaque(size = 8, align = 2))]
///         #[opaque(size = 16, align = 8)]
///         pub type Context: Copy;
///
///         /// Returns a fresh checksum state.
///         #[symbol = "_cksum_cfg_init"]
///         fn init() -> Self::Context;
///     }
///
///     /// The global checksum.
///     pub struct Checksum;
///
///     /// Set the global checksum implementation.
///     macro checksum_impl(path = $crate);
/// }
/// # struct Fletcher;
/// # impl ChecksumDriver for Fletcher {
/// #     type Context = (u32, u32);
/// #     fn init() -> (u32, u32) { (0, 0) }
/// # }
/// # checksum_impl!(Fletcher);
/// # fn main() { let _ = Checksum::init(); }
/// ```
///
/// The attributes are tried in source order and the first one whose predicate holds is
/// used, so several predicates may hold at once. A plain `#[opaque]` is the fallback for
/// when none holds; it must come last, and may be omitted, in which case no predicate
/// holding is a compile error. Only `opaque` may be placed under `cfg_attr`. The predicates
/// are evaluated in the crate defining the unitrait, and implementations check their type
/// against whichever layout was chosen there.
///
/// The expansion selects the layout with [`core::cfg_select!`], available since Rust
/// 1.95; declarations without `cfg_attr` don't use it. The expansion grows linearly with
/// the number of attributes.
///
/// # Dropping opaque types
///
/// The caller can't see the implementation's associated type, so it can't drop it either.
/// Whether an opaque type has drop glue at all is declared by a `Drop` bound:
///
/// - **With** a `Drop` bound, the opaque struct gets a `Drop` impl, which drops the
///   implementation's value in place through an extern symbol. Such a declaration must carry
///   a `#[drop_symbol = "..."]` attribute naming that symbol. The implementation may pick an
///   associated type that needs dropping, or one that doesn't.
/// - **Without** one, the opaque struct has no `Drop` impl, dropping one does nothing, and
///   the declaration must not carry a `#[drop_symbol = "..."]` attribute. The implementation
///   macro checks at compile time that the implementation's associated type has no drop glue
///   ([`core::mem::needs_drop`] is `false`), so nothing is silently leaked. That is stricter
///   than the type having no `Drop` impl: it also rejects a type that merely *contains*
///   something needing drop.
///
/// Leaving the `Drop` bound off is the right choice for plain-data contexts such as an index
/// or a handle: it reserves no symbol name, and it keeps the opaque struct free of drop glue.
///
/// `Drop` is written like the [marker bounds](#marker-bounds-on-opaque-types) but isn't one:
/// it is never emitted, neither as a bound on the associated type (implementations are free
/// to pick a type that implements `Drop` or not) nor as an `impl` on the opaque struct
/// (which gets its `Drop` impl from the drop symbol). It is mutually exclusive with `Copy`.
/// Adding or removing it is an ABI-breaking change, since the two sides would then disagree
/// on whether the drop symbol is exported and called.
///
/// Methods may use `Self::Name` in any parameter and in the return type, in these forms:
///
/// - `Self::Name` (by value): the dispatch method takes or returns the opaque struct by
///   value. A by-value parameter consumes the value, and it is dropped (or kept) by the
///   implementation.
/// - `&Self::Name` and `&mut Self::Name` (parameters only): the dispatch method takes a
///   reference to the opaque struct.
/// - `Pin<&Self::Name>` and `Pin<&mut Self::Name>` (parameters only): the dispatch method
///   takes a pinned reference to the opaque struct, and the implementation receives a
///   pinned reference to its own value, at the address the caller pinned.
///
/// Any of these may also be nested, to any depth, inside `Option`, `Result`, tuples and
/// arrays, as in `Option<Self::Name>`, `Result<(Self::A, u32), Self::B>`,
/// `[Option<&mut Self::Name>; 4]` or `Option<Pin<&mut Self::Name>>`. The dispatch method
/// takes or returns the same composite of the opaque structs, and the implementation macro
/// takes the composite apart and rebuilds it around the converted values: an `Option` costs
/// a `match` and an array a `map`, on top of the moves a by-value opaque costs anyway. A
/// composite containing a borrowed opaque type is, like the borrow itself, parameter-only.
///
/// Write `Option`, `Result` and `Pin` by their bare names. Around an opaque type they always
/// mean [`core::option::Option`], [`core::result::Result`] and [`core::pin::Pin`], whatever
/// those names happen to be in scope, and are rewritten to their absolute paths everywhere
/// they're emitted, the trait itself included: an implementation in a scope that shadows
/// one of them must spell out the absolute path. Away from opaque types, a signature is
/// emitted exactly as written and means whatever it means in scope, so implementations are
/// ordinary safe code.
///
/// Anything else is rejected at compile time, since it can't be bridged across the extern
/// symbol: opaque types inside other generic types (`Box<Self::Name>`, `Vec<Self::Name>`, a
/// generic type of your own), and references to composites (`&[Self::Name]`,
/// `&Option<Self::Name>`, `&mut [Self::Name; 4]`). The opaque struct is only an upper bound
/// on the implementation's type, so a slice or array of one doesn't have the stride of a
/// slice or array of the other, and `Option`s and tuples of the two need not share a layout
/// at all. Pass such composites by value instead.
///
/// # Marker bounds on opaque types
///
/// The caller can't see the implementation's associated type, so the opaque struct can't
/// derive its auto traits from it. Instead, the opaque struct implements *no* auto trait by
/// default, and each one is opted into by declaring it as a bound on the associated type:
///
/// ```
/// unitrait::unitrait! {
///     pub trait SessionDriver {
///         /// Can be moved between threads, but not shared, and must not be moved once
///         /// it has been pinned. Duplicating one duplicates the implementation's state.
///         #[opaque(size = 64, align = 8)]
///         #[drop_symbol = "_session_state_drop"]
///         #[clone_symbol = "_session_state_clone"]
///         pub type State: Send + Clone + Drop;
///
///         /// A plain copyable handle: no drop glue, and duplicating it is free.
///         #[opaque(size = 4, align = 4)]
///         pub type Id: Copy + Send + Sync;
///
///         #[symbol = "_session_open"]
///         fn open() -> Self::State;
///
///         #[symbol = "_session_id"]
///         fn id(state: &Self::State) -> Self::Id;
///     }
///
///     pub struct Session;
///
///     macro session_impl(path = $crate);
/// }
/// # struct MySession;
/// # impl SessionDriver for MySession {
/// #     type State = u64;
/// #     type Id = u32;
/// #     fn open() -> u64 { 7 }
/// #     fn id(state: &u64) -> u32 { *state as u32 }
/// # }
/// # session_impl!(MySession);
/// # fn main() { let s = Session::open(); assert_eq!(core::mem::size_of_val(&Session::id(&s)), 4); }
/// ```
///
/// The supported marker bounds are `Send`, `Sync`, `Unpin`, `UnwindSafe`, `RefUnwindSafe`,
/// `Copy` and `Clone`, alongside the `Drop` bound described
/// [above](#dropping-opaque-types). Each marker is emitted both as a bound on the associated
/// type — so the compiler rejects an implementation whose type doesn't implement it — and as
/// an `impl` on the opaque struct, which is therefore never more permissive than the
/// implementation's own type. They must be written by their bare names; they always mean the
/// `core` traits, so a trait of the same name in scope where `unitrait!` is invoked changes
/// nothing, and no other bound (including lifetimes and `?Sized`) is accepted.
///
/// `Copy` implies the implementation's associated type has no drop glue, so it is mutually
/// exclusive with the `Drop` bound; the opaque struct couldn't implement both anyway.
/// `Clone` is implemented alongside `Copy`, by copying the bytes.
///
/// `Clone` on its own is the one marker whose `impl` needs the implementation's help: the
/// caller can't duplicate a value of a type it can't see. A `Clone` declaration must
/// therefore carry a `#[clone_symbol = "..."]` attribute, and `<OpaqueType as
/// Clone>::clone` dispatches through that symbol, which the implementation macro exports as
/// a function cloning the implementation's value. `clone_from` is left at its default, so it
/// goes through `clone` too. `Clone` is mutually exclusive with `Copy`, which already
/// provides a (trivial) `Clone` impl.
///
/// Not declaring a bound is what makes the other features sound. In particular, an opaque
/// type without `Unpin` can only be placed behind a `Pin` by actually pinning it, which is
/// what lets `Pin<&mut Self::Name>` parameters hand the implementation a genuinely pinned
/// value.
///
/// # Implementing a unitrait
///
/// Implement the trait on some type (typically a unit struct), then invoke the
/// implementation macro on that type:
///
/// ```ignore
/// struct MyDriver;
///
/// impl frob::Driver for MyDriver {
///     fn level() -> u32 { 42 }
///     unsafe fn item(token: u32) -> &'static mut frob::Item { ... }
/// }
///
/// frob::frob_driver_impl!(MyDriver);
/// ```
///
/// The dispatch type implements the trait too, but it is the *caller* side: registering it as
/// the implementation would make every call loop back into itself.
///
/// When implementing from within the defining crate itself, the implementation macro must be
/// invoked by its bare name (textually scoped, like all `macro_rules!` macros), not through a
/// `crate::` path: `#[macro_use]` on the defining module can be used to extend its textual
/// scope to the rest of the crate.
///
/// # Name resolution in method signatures
///
/// The method signatures are pasted verbatim both into the defining crate (for the trait and
/// the dispatch type) and into implementor crates (by the implementation macro). Therefore, the
/// types they name must resolve in both places. The implementation macro expands inside a
/// scope with `use PATH::*;`, so this works out as long as every type named in the signatures
/// is:
///
/// - a primitive, or
/// - an absolute path (`core::...`, `::somecrate::...`), or
/// - publicly reachable at the root of the module containing the `unitrait!` invocation, and
///   named unqualified (like `Item` in the example above). If the type is defined elsewhere,
///   re-export it with `pub use`.
///
/// That glob is confined to the shims: the type passed to the implementation macro is
/// resolved in the caller's own scope, so it may share its name with a public item of the
/// defining module (the dispatch type, say) without being confused with it.
///
/// # Linkage details
///
/// Calls from the dispatch type to the implementation are done via `extern "Rust"` functions.
///
/// For each method, the dispatch method calls `extern "Rust" { fn "SYMBOL"(...); }`, and the
/// implementation macro exports `#[export_name = "SYMBOL"] fn(...)` which forwards to the
/// trait implementation. The linker resolves the former to the latter. If no crate in the tree
/// invokes the implementation macro, linking fails with an "undefined symbol" error; if more
/// than one does, it fails with a "duplicate symbol" error.
///
/// For opaque associated types, both sides of the symbol pass the opaque struct; the exported
/// functions cast it to the implementation's associated type (sound thanks to the size/align
/// checks and the always-initialized invariant), taking apart and rebuilding any `Option`,
/// `Result`, tuple or array around it. For each opaque type with a `Drop` bound,
/// the implementation macro additionally exports its drop symbol, as a function dropping the
/// implementation's value in place; the opaque struct's `Drop` impl calls it. A `Clone` bound
/// works the same way, with a function cloning the implementation's value.
///
/// Symbols derived from a `#[symbol_prefix = "..."]` are exactly as real as explicitly named
/// ones: nothing about the prefix survives into the generated code beyond the names it
/// produces.
///
/// Because the contract between the two sides is just the symbol name and signature, crates
/// may define and implement the unitrait through *different versions* of the defining crate
/// (as happens during major-version transitions of the defining crate) and still link
/// correctly, as long as the symbol names and signatures match. For opaque types, the size
/// and alignment are part of that ABI contract too: shrinking either is ABI-breaking, and
/// so is changing the declared bounds, since the two sides would then disagree on what the
/// opaque struct implements and on whether it is dropped through a symbol.
#[proc_macro]
pub fn unitrait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as UnitraitInput);
    expand(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

struct UnitraitInput {
    trait_docs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    opaques: Vec<OpaqueDecl>,
    methods: Vec<Method>,
    /// Attributes written on the `struct` line, applied verbatim to the dispatch type.
    dispatch_attrs: Vec<Attribute>,
    /// The dispatch type's visibility, which its inherent methods share.
    dispatch_vis: Visibility,
    /// The dispatch type's name.
    dispatch: Ident,
    mac_docs: Vec<Attribute>,
    mac_name: Ident,
    path: TokenStream,
}

struct OpaqueDecl {
    docs: Vec<Attribute>,
    vis: Visibility,
    /// The associated type's name, e.g. `Context`.
    assoc: Ident,
    /// The generated opaque struct's name: dispatch type name + associated type name.
    opaque: Ident,
    /// The `#[opaque(size = N, align = M)]` attributes, in source order. Non-empty; at most
    /// the last one is unconditional.
    layouts: Vec<OpaqueLayout>,
    /// The marker traits declared as bounds on the associated type. The `Drop` bound is not
    /// one of them: it's recorded by `drop_symbol` instead.
    bounds: Vec<Marker>,
    /// The symbol of the function dropping the value in place. `Some` exactly when the
    /// declaration has a `Drop` bound, which also requires the `#[drop_symbol = "..."]`
    /// attribute; otherwise the opaque struct has no drop glue and the implementation's
    /// associated type is required to have none either.
    drop_symbol: Option<LitStr>,
    /// The symbol of the function cloning the value. `Some` exactly when the declaration
    /// has a `Clone` bound, which also requires the `#[clone_symbol = "..."]` attribute.
    clone_symbol: Option<LitStr>,
}

/// A marker trait that may be declared as a bound on an opaque associated type.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Marker {
    Send,
    Sync,
    Unpin,
    UnwindSafe,
    RefUnwindSafe,
    Copy,
    Clone,
}

const MARKERS: &[(&str, Marker)] = &[
    ("Send", Marker::Send),
    ("Sync", Marker::Sync),
    ("Unpin", Marker::Unpin),
    ("UnwindSafe", Marker::UnwindSafe),
    ("RefUnwindSafe", Marker::RefUnwindSafe),
    ("Copy", Marker::Copy),
    ("Clone", Marker::Clone),
];

const BOUND_HELP: &str = "only `Send`, `Sync`, `Unpin`, `UnwindSafe`, `RefUnwindSafe`, `Copy`, `Clone` and `Drop` are allowed as bounds on an opaque associated type, written by their bare names";

impl Marker {
    fn from_ident(ident: &Ident) -> Option<Marker> {
        MARKERS.iter().find(|(n, _)| ident == n).map(|&(_, m)| m)
    }

    fn name(self) -> &'static str {
        MARKERS.iter().find(|&&(_, m)| m == self).unwrap().0
    }

    /// The absolute path of the trait.
    ///
    /// Bounds are matched by their bare name and always mean the `core` trait, so a trait
    /// of the same name in scope where `unitrait!` is invoked can't make the bound on the
    /// associated type and the `impl` on the opaque struct refer to different traits.
    fn path(self) -> TokenStream {
        match self {
            Marker::Send => quote!(::core::marker::Send),
            Marker::Sync => quote!(::core::marker::Sync),
            Marker::Unpin => quote!(::core::marker::Unpin),
            Marker::UnwindSafe => quote!(::core::panic::UnwindSafe),
            Marker::RefUnwindSafe => quote!(::core::panic::RefUnwindSafe),
            Marker::Copy => quote!(::core::marker::Copy),
            Marker::Clone => quote!(::core::clone::Clone),
        }
    }
}

/// The parsed `: Send + Sync + Drop` bounds of an opaque associated type declaration.
struct Bounds {
    /// The marker traits, which are emitted both as bounds on the associated type and as
    /// `impl`s on the opaque struct.
    markers: Vec<Marker>,
    /// The span of the `Drop` bound, if written. Unlike the markers it is not a real trait
    /// bound: it declares that the opaque struct has drop glue.
    drop_bound: Option<Span>,
    /// The span of the `Clone` marker, if written, for error reporting.
    clone_bound: Option<Span>,
}

/// Parses the optional `: Send + Sync + Drop` bounds of an opaque associated type
/// declaration.
fn parse_bounds(input: ParseStream) -> syn::Result<Bounds> {
    let mut markers = vec![];
    let mut drop_bound = None;
    let mut clone_bound = None;
    if !input.peek(Token![:]) {
        return Ok(Bounds {
            markers,
            drop_bound,
            clone_bound,
        });
    }
    input.parse::<Token![:]>()?;
    let parsed = Punctuated::<TypeParamBound, Token![+]>::parse_separated_nonempty(input)?;
    for bound in &parsed {
        let TypeParamBound::Trait(t) = bound else {
            return Err(syn::Error::new(bound.span(), BOUND_HELP));
        };
        let plain = matches!(t.modifier, TraitBoundModifier::None)
            && t.lifetimes.is_none()
            && t.path.leading_colon.is_none()
            && t.path.segments.len() == 1
            && t.path.segments[0].arguments.is_none();
        if !plain {
            return Err(syn::Error::new(bound.span(), BOUND_HELP));
        }
        let ident = &t.path.segments[0].ident;
        // `Drop` is accepted where a bound goes, but it isn't one: it's never emitted, on
        // the associated type or on the opaque struct. It only says the opaque struct has
        // drop glue, and therefore that the implementation's type may need dropping.
        if ident == "Drop" {
            if drop_bound.is_some() {
                return Err(syn::Error::new(ident.span(), "duplicate `Drop` bound"));
            }
            drop_bound = Some(ident.span());
            continue;
        }
        let Some(marker) = Marker::from_ident(ident) else {
            return Err(syn::Error::new(ident.span(), BOUND_HELP));
        };
        if markers.contains(&marker) {
            return Err(syn::Error::new(
                ident.span(),
                format!("duplicate `{}` bound", marker.name()),
            ));
        }
        if marker == Marker::Clone {
            clone_bound = Some(ident.span());
        }
        markers.push(marker);
    }
    if let Some(span) = drop_bound
        && markers.contains(&Marker::Copy)
    {
        return Err(syn::Error::new(
            span,
            "`Copy` and `Drop` are mutually exclusive: a `Copy` type never has drop glue",
        ));
    }
    if let Some(span) = clone_bound
        && markers.contains(&Marker::Copy)
    {
        return Err(syn::Error::new(
            span,
            "`Copy` and `Clone` are mutually exclusive: a `Copy` opaque type already implements `Clone` by copying its bytes",
        ));
    }
    Ok(Bounds {
        markers,
        drop_bound,
        clone_bound,
    })
}

struct Method {
    docs: Vec<Attribute>,
    unsafety: Option<Token![unsafe]>,
    name: Ident,
    symbol: LitStr,
    args: Vec<(Ident, Type)>,
    ret: Option<Type>,
}

/// One `#[opaque(size = N, align = M)]` attribute, possibly wrapped in a `cfg_attr`.
struct OpaqueLayout {
    /// The `cfg_attr` predicate, verbatim; `None` for a plain `#[opaque]`.
    cfg: Option<TokenStream>,
    /// The declared alignment, as written.
    align: LitInt,
    /// The declared size rounded up to a multiple of the alignment: the opaque struct's
    /// storage. Padding the storage itself, rather than letting `repr(align)` pad the
    /// struct, keeps every byte of the struct inside `_data`, so the implementation's value
    /// may use all of `size_of::<Opaque>()` and the implementation macro can check against
    /// that instead of the literals, which it can't see when they depend on `cfg`s.
    padded_size: LitInt,
}

struct SplitAttrs {
    docs: Vec<Attribute>,
    symbol: Option<LitStr>,
    drop_symbol: Option<LitStr>,
    clone_symbol: Option<LitStr>,
    /// The `#[opaque]` attributes in source order, see [`OpaqueDecl::layouts`].
    opaque: Vec<OpaqueLayout>,
}

/// Parses the string literal of a `#[name = "..."]` attribute.
fn parse_str_attr(attr: &Attribute, name: &str) -> syn::Result<LitStr> {
    let syn::Meta::NameValue(nv) = &attr.meta else {
        return Err(syn::Error::new(
            attr.span(),
            format!("expected `#[{name} = \"...\"]`"),
        ));
    };
    let Expr::Lit(l) = &nv.value else {
        return Err(syn::Error::new(
            nv.value.span(),
            "expected a string literal",
        ));
    };
    let Lit::Str(s) = &l.lit else {
        return Err(syn::Error::new(l.span(), "expected a string literal"));
    };
    Ok(s.clone())
}

const CFG_ATTR_HELP: &str = "expected `#[cfg_attr(predicate, opaque(size = N, align = M))]`";

/// Parses the `size = N, align = M` arguments of an `#[opaque(...)]` attribute, either
/// written directly or under the `cfg_attr` predicate `cfg`.
fn parse_opaque_args(list: &syn::MetaList, cfg: Option<TokenStream>) -> syn::Result<OpaqueLayout> {
    let mut size = None;
    let mut align = None;
    list.parse_nested_meta(|meta| {
        let lit: LitInt = meta.value()?.parse()?;
        if meta.path.is_ident("size") {
            size = Some(lit);
        } else if meta.path.is_ident("align") {
            align = Some(lit);
        } else {
            return Err(meta.error("expected `size` or `align`"));
        }
        Ok(())
    })?;
    let (Some(size), Some(align)) = (size, align) else {
        return Err(syn::Error::new(
            list.span(),
            "expected `#[opaque(size = N, align = M)]`",
        ));
    };
    let align_value: usize = align.base10_parse()?;
    if !align_value.is_power_of_two() {
        return Err(syn::Error::new(
            align.span(),
            "invalid alignment value: not a power of two",
        ));
    }
    let size_value: usize = size.base10_parse()?;
    let Some(padded) = size_value.checked_next_multiple_of(align_value) else {
        return Err(syn::Error::new(
            size.span(),
            "size rounded up to a multiple of the alignment overflows `usize`",
        ));
    };
    Ok(OpaqueLayout {
        cfg,
        align,
        padded_size: LitInt::new(&padded.to_string(), size.span()),
    })
}

/// Splits attributes into doc comments, an optional `#[symbol = "..."]`, optional
/// `#[drop_symbol = "..."]` and `#[clone_symbol = "..."]`, and the
/// `#[opaque(size = N, align = M)]` attributes, each optionally under a `cfg_attr`.
fn split_attrs(attrs: Vec<Attribute>) -> syn::Result<SplitAttrs> {
    let mut docs = vec![];
    let mut symbol = None;
    let mut drop_symbol = None;
    let mut clone_symbol = None;
    let mut opaque: Vec<OpaqueLayout> = vec![];
    for attr in attrs {
        if attr.path().is_ident("doc") {
            docs.push(attr);
        } else if attr.path().is_ident("symbol") {
            if symbol.is_some() {
                return Err(syn::Error::new(
                    attr.span(),
                    "duplicate `#[symbol]` attribute",
                ));
            }
            symbol = Some(parse_str_attr(&attr, "symbol")?);
        } else if attr.path().is_ident("drop_symbol") {
            if drop_symbol.is_some() {
                return Err(syn::Error::new(
                    attr.span(),
                    "duplicate `#[drop_symbol]` attribute",
                ));
            }
            drop_symbol = Some(parse_str_attr(&attr, "drop_symbol")?);
        } else if attr.path().is_ident("clone_symbol") {
            if clone_symbol.is_some() {
                return Err(syn::Error::new(
                    attr.span(),
                    "duplicate `#[clone_symbol]` attribute",
                ));
            }
            clone_symbol = Some(parse_str_attr(&attr, "clone_symbol")?);
        } else if attr.path().is_ident("opaque") {
            if opaque.iter().any(|l| l.cfg.is_none()) {
                return Err(syn::Error::new(
                    attr.span(),
                    "duplicate `#[opaque]` attribute",
                ));
            }
            opaque.push(parse_opaque_args(attr.meta.require_list()?, None)?);
        } else if attr.path().is_ident("cfg_attr") {
            // `cfg_attr(predicate, attr, attr, ...)`. The predicate is kept verbatim, to be
            // re-emitted as a `cfg_select!` arm.
            let (cfg, metas) = attr.parse_args_with(|input: ParseStream| {
                let mut cfg = TokenStream::new();
                while !input.is_empty() && !input.peek(Token![,]) {
                    cfg.extend([input.parse::<TokenTree>()?]);
                }
                if cfg.is_empty() || input.is_empty() {
                    return Err(input.error(CFG_ATTR_HELP));
                }
                input.parse::<Token![,]>()?;
                let metas = Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;
                Ok((cfg, metas))
            })?;
            if metas.is_empty() {
                return Err(syn::Error::new(attr.span(), CFG_ATTR_HELP));
            }
            let mut seen = false;
            for meta in metas {
                let list = match &meta {
                    syn::Meta::List(list) if list.path.is_ident("opaque") => list,
                    _ => {
                        return Err(syn::Error::new(
                            meta.span(),
                            "only `opaque(size = N, align = M)` may be placed under `cfg_attr`",
                        ));
                    }
                };
                if seen {
                    return Err(syn::Error::new(
                        meta.span(),
                        "duplicate `#[opaque]` attribute",
                    ));
                }
                seen = true;
                // The first `#[opaque]` whose predicate holds wins, so anything after an
                // unconditional one can never apply.
                if opaque.iter().any(|l| l.cfg.is_none()) {
                    return Err(syn::Error::new(
                        meta.span(),
                        "this `#[opaque]` can never apply: the unconditional `#[opaque]` attribute above it always applies first; move it above",
                    ));
                }
                opaque.push(parse_opaque_args(list, Some(cfg.clone()))?);
            }
        } else {
            return Err(syn::Error::new(
                attr.span(),
                "unexpected attribute; expected doc comments, `#[symbol = \"...\"]`, `#[drop_symbol = \"...\"]`, `#[clone_symbol = \"...\"]`, `#[opaque(size = N, align = M)]` or `#[cfg_attr(predicate, opaque(size = N, align = M))]`",
            ));
        }
    }
    Ok(SplitAttrs {
        docs,
        symbol,
        drop_symbol,
        clone_symbol,
        opaque,
    })
}

/// Splits the trait's attributes into doc comments and an optional
/// `#[symbol_prefix = "..."]`.
fn split_trait_attrs(attrs: Vec<Attribute>) -> syn::Result<(Vec<Attribute>, Option<LitStr>)> {
    let mut docs = vec![];
    let mut prefix = None;
    for attr in attrs {
        if attr.path().is_ident("doc") {
            docs.push(attr);
        } else if attr.path().is_ident("symbol_prefix") {
            if prefix.is_some() {
                return Err(syn::Error::new(
                    attr.span(),
                    "duplicate `#[symbol_prefix]` attribute",
                ));
            }
            prefix = Some(parse_str_attr(&attr, "symbol_prefix")?);
        } else {
            return Err(syn::Error::new(
                attr.span(),
                "unexpected attribute; expected doc comments or `#[symbol_prefix = \"...\"]`",
            ));
        }
    }
    Ok((docs, prefix))
}

/// The symbol for an opaque associated type's `drop` or `clone` function, derived from the
/// trait's `#[symbol_prefix = "..."]`. Fails with `missing` if there is no prefix, spanned at
/// the bound that requires the symbol.
fn assoc_symbol(
    prefix: &Option<LitStr>,
    assoc: &Ident,
    what: &str,
    span: Span,
    missing: &str,
) -> syn::Result<LitStr> {
    match prefix {
        Some(prefix) => Ok(LitStr::new(
            &format!("{}_{assoc}_{what}", prefix.value()),
            span,
        )),
        None => Err(syn::Error::new(span, missing)),
    }
}

fn only_docs(attrs: Vec<Attribute>) -> syn::Result<Vec<Attribute>> {
    if let Some(attr) = attrs.iter().find(|a| !a.path().is_ident("doc")) {
        return Err(syn::Error::new(
            attr.span(),
            "only doc comments are allowed here",
        ));
    }
    Ok(attrs)
}

impl Parse for UnitraitInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (trait_docs, symbol_prefix) = split_trait_attrs(input.call(Attribute::parse_outer)?)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![trait]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);

        let mut opaques: Vec<OpaqueDecl> = vec![];
        let mut methods: Vec<Method> = vec![];
        while !content.is_empty() {
            let attrs = content.call(Attribute::parse_outer)?;
            let ivis: Visibility = content.parse()?;
            if content.peek(Token![type]) {
                content.parse::<Token![type]>()?;
                let assoc: Ident = content.parse()?;
                let Bounds {
                    markers,
                    drop_bound,
                    clone_bound,
                } = parse_bounds(&content)?;
                if content.peek(Token![=]) {
                    return Err(content.error(format!(
                        "the opaque type's name is derived automatically as the dispatch type's name followed by `{assoc}`; remove the `= ...`"
                    )));
                }
                content.parse::<Token![;]>()?;
                let SplitAttrs {
                    docs,
                    symbol,
                    drop_symbol,
                    clone_symbol,
                    opaque: layouts,
                } = split_attrs(attrs)?;
                if let Some(symbol) = symbol {
                    return Err(syn::Error::new(
                        symbol.span(),
                        "opaque associated types name their drop symbol with `#[drop_symbol = \"...\"]`; `#[symbol = \"...\"]` is only allowed on methods",
                    ));
                }
                // The `Drop` bound decides whether the opaque struct has drop glue; the
                // attribute only names the symbol to drop through. The two must agree.
                let drop_symbol = match (drop_bound, drop_symbol) {
                    (Some(_), symbol @ Some(_)) => symbol,
                    (None, None) => None,
                    (Some(span), None) => Some(assoc_symbol(
                        &symbol_prefix,
                        &assoc,
                        "drop",
                        span,
                        "an opaque associated type with a `Drop` bound requires a `#[drop_symbol = \"...\"]` attribute naming the extern symbol for dropping the value in place, or a `#[symbol_prefix = \"...\"]` attribute on the trait to derive one",
                    )?),
                    (None, Some(drop_symbol)) => {
                        return Err(syn::Error::new(
                            drop_symbol.span(),
                            "an opaque associated type without a `Drop` bound has no drop glue, so it must not have a `#[drop_symbol = \"...\"]` attribute; add a `Drop` bound to give it one",
                        ));
                    }
                };
                // Same for `Clone`: the bound decides, the attribute names the symbol.
                let clone_symbol = match (clone_bound, clone_symbol) {
                    (Some(_), symbol @ Some(_)) => symbol,
                    (None, None) => None,
                    (Some(span), None) => Some(assoc_symbol(
                        &symbol_prefix,
                        &assoc,
                        "clone",
                        span,
                        "an opaque associated type with a `Clone` bound requires a `#[clone_symbol = \"...\"]` attribute naming the extern symbol for cloning the value, or a `#[symbol_prefix = \"...\"]` attribute on the trait to derive one",
                    )?),
                    (None, Some(clone_symbol)) => {
                        return Err(syn::Error::new(
                            clone_symbol.span(),
                            "an opaque associated type without a `Clone` bound can't be cloned, so it must not have a `#[clone_symbol = \"...\"]` attribute; add a `Clone` bound to give it one",
                        ));
                    }
                };
                if layouts.is_empty() {
                    return Err(syn::Error::new(
                        assoc.span(),
                        "opaque associated types require an `#[opaque(size = N, align = M)]` attribute",
                    ));
                }
                if opaques.iter().any(|o| o.assoc == assoc) {
                    return Err(syn::Error::new(assoc.span(), "duplicate associated type"));
                }
                // Named after the dispatch type, which is parsed after the trait body; filled
                // in below.
                let opaque = assoc.clone();
                opaques.push(OpaqueDecl {
                    docs,
                    vis: ivis,
                    assoc,
                    opaque,
                    layouts,
                    bounds: markers,
                    drop_symbol,
                    clone_symbol,
                });
            } else {
                if !matches!(ivis, Visibility::Inherited) {
                    return Err(syn::Error::new(
                        ivis.span(),
                        "unitrait methods must not have a visibility; they are as visible as the dispatch type, so set the visibility on the `struct` line instead",
                    ));
                }
                let unsafety: Option<Token![unsafe]> = content.parse()?;
                content.parse::<Token![fn]>()?;
                let fname: Ident = content.parse()?;
                let args_content;
                parenthesized!(args_content in content);
                let mut args = vec![];
                while !args_content.is_empty() {
                    if args_content.peek(Token![self])
                        || (args_content.peek(Token![&])
                            && (args_content.peek2(Token![self])
                                || (args_content.peek2(Token![mut])
                                    && args_content.peek3(Token![self]))))
                    {
                        return Err(args_content.error("unitrait methods must not have a `self` parameter; store state in `static`s or in an opaque associated type"));
                    }
                    let arg: Ident = args_content.parse()?;
                    args_content.parse::<Token![:]>()?;
                    let ty: Type = args_content.parse()?;
                    args.push((arg, ty));
                    if !args_content.is_empty() {
                        args_content.parse::<Token![,]>()?;
                    }
                }
                let ret = if content.peek(Token![->]) {
                    content.parse::<Token![->]>()?;
                    Some(content.parse::<Type>()?)
                } else {
                    None
                };
                content.parse::<Token![;]>()?;
                let SplitAttrs {
                    docs,
                    symbol,
                    drop_symbol,
                    clone_symbol,
                    opaque,
                } = split_attrs(attrs)?;
                if !opaque.is_empty() {
                    return Err(syn::Error::new(
                        fname.span(),
                        "`#[opaque]` is only allowed on associated type declarations",
                    ));
                }
                if let Some(drop_symbol) = drop_symbol {
                    return Err(syn::Error::new(
                        drop_symbol.span(),
                        "`#[drop_symbol]` is only allowed on opaque associated type declarations",
                    ));
                }
                if let Some(clone_symbol) = clone_symbol {
                    return Err(syn::Error::new(
                        clone_symbol.span(),
                        "`#[clone_symbol]` is only allowed on opaque associated type declarations",
                    ));
                }
                let symbol = match symbol {
                    Some(symbol) => symbol,
                    None => match &symbol_prefix {
                        Some(prefix) => {
                            LitStr::new(&format!("{}_{fname}", prefix.value()), fname.span())
                        }
                        None => {
                            return Err(syn::Error::new(
                                fname.span(),
                                "unitrait methods require a `#[symbol = \"...\"]` attribute, or a `#[symbol_prefix = \"...\"]` attribute on the trait to derive one from the method name",
                            ));
                        }
                    },
                };
                methods.push(Method {
                    docs,
                    unsafety,
                    name: fname,
                    symbol,
                    args,
                    ret,
                });
            }
        }

        let dispatch_attrs = input.call(Attribute::parse_outer)?;
        let dispatch_vis: Visibility = input.parse()?;
        if !input.peek(Token![struct]) {
            return Err(input.error(
                "expected `struct NAME;`, naming the dispatch type callers use to reach the global implementation",
            ));
        }
        input.parse::<Token![struct]>()?;
        let dispatch: Ident = input.parse()?;
        input.parse::<Token![;]>()?;
        if dispatch == name {
            return Err(syn::Error::new(
                dispatch.span(),
                "the dispatch type can't have the same name as the trait",
            ));
        }
        for o in &mut opaques {
            o.opaque = format_ident!("{dispatch}{}", o.assoc, span = o.assoc.span());
        }

        let mac_docs = only_docs(input.call(Attribute::parse_outer)?)?;
        input.parse::<Token![macro]>()?;
        let mac_name: Ident = input.parse()?;
        let mac_args;
        parenthesized!(mac_args in input);
        let path_kw: Ident = mac_args.parse()?;
        if path_kw != "path" {
            return Err(syn::Error::new(path_kw.span(), "expected `path = ...`"));
        }
        mac_args.parse::<Token![=]>()?;
        let path: TokenStream = mac_args.parse()?;
        if path.is_empty() {
            return Err(mac_args.error("expected a path starting with `$crate`"));
        }
        input.parse::<Token![;]>()?;

        Ok(UnitraitInput {
            trait_docs,
            vis,
            name,
            opaques,
            methods,
            dispatch_attrs,
            dispatch_vis,
            dispatch,
            mac_docs,
            mac_name,
            path,
        })
    }
}

/// How a method uses a type slot (one parameter, or the return type), as far as opaque
/// associated types are concerned.
///
/// A composite is only built when an opaque associated type appears somewhere inside it; a
/// type with no `Self` in it is [`Plain`](Shape::Plain), however it is spelled.
enum Shape {
    /// A type containing no `Self`, passed through verbatim.
    Plain(Type),
    /// An opaque associated type, by value or borrowed.
    Opaque {
        /// Index into [`UnitraitInput::opaques`].
        index: usize,
        mode: Mode,
        /// The span of the use, for errors about it.
        span: Span,
    },
    /// `Option<T>`, with an opaque associated type somewhere in `T`.
    Option(Span, Box<Shape>),
    /// `Result<T, E>`, with an opaque associated type somewhere in `T` or `E`.
    Result(Span, Box<Shape>, Box<Shape>),
    /// A tuple, with an opaque associated type somewhere in an element.
    Tuple(Span, Vec<Shape>),
    /// `[T; N]`, with an opaque associated type somewhere in `T`. The length is kept as
    /// written.
    Array(Span, Box<Shape>, Expr),
}

/// How an opaque associated type is used at a leaf of a [`Shape`].
#[derive(Clone, Copy)]
enum Mode {
    /// `Self::Name`.
    Value,
    /// `&Self::Name`.
    Ref,
    /// `&mut Self::Name`.
    RefMut,
    /// `Pin<&Self::Name>`.
    PinRef,
    /// `Pin<&mut Self::Name>`.
    PinRefMut,
}

impl Shape {
    /// The span of the first borrowed opaque associated type (`&`, `&mut` or `Pin`) in the
    /// shape. A borrow can't be returned: the implementation's value is smaller than the
    /// opaque struct in general, so a reference to it can't be read as one to the opaque
    /// struct, and there'd be nothing to tie its lifetime to anyway.
    fn find_borrow(&self) -> Option<Span> {
        match self {
            Shape::Plain(_)
            | Shape::Opaque {
                mode: Mode::Value, ..
            } => None,
            Shape::Opaque { span, .. } => Some(*span),
            Shape::Option(_, inner) | Shape::Array(_, inner, _) => inner.find_borrow(),
            Shape::Result(_, ok, err) => ok.find_borrow().or_else(|| err.find_borrow()),
            Shape::Tuple(_, elems) => elems.iter().find_map(Shape::find_borrow),
        }
    }

    /// Renders the shape as a type, spelling each opaque leaf with `leaf` (given its index
    /// and the span of its use) wrapped in `&`, `&mut` or `Pin` as the leaf's mode says.
    ///
    /// `Option`, `Result` and `Pin` are always spelled by their absolute `core` paths, so a
    /// type of the same name in scope in either the defining or the implementing crate
    /// can't change what the generated code means. Plain types are emitted verbatim.
    /// Everything is emitted at the span of the type it was written as, so compiler
    /// diagnostics about a signature point into the trait as written.
    fn render(&self, leaf: &dyn Fn(usize, Span) -> TokenStream) -> TokenStream {
        match self {
            Shape::Plain(ty) => ty.to_token_stream(),
            Shape::Opaque { index, mode, span } => {
                let t = leaf(*index, *span);
                match mode {
                    Mode::Value => t,
                    Mode::Ref => quote_spanned!(*span=> &#t),
                    Mode::RefMut => quote_spanned!(*span=> &mut #t),
                    Mode::PinRef => quote_spanned!(*span=> ::core::pin::Pin<&#t>),
                    Mode::PinRefMut => quote_spanned!(*span=> ::core::pin::Pin<&mut #t>),
                }
            }
            Shape::Option(span, inner) => {
                let inner = inner.render(leaf);
                quote_spanned!(*span=> ::core::option::Option<#inner>)
            }
            Shape::Result(span, ok, err) => {
                let (ok, err) = (ok.render(leaf), err.render(leaf));
                quote_spanned!(*span=> ::core::result::Result<#ok, #err>)
            }
            Shape::Tuple(span, elems) => {
                let elems = elems.iter().map(|e| e.render(leaf));
                quote_spanned!(*span=> (#(#elems,)*))
            }
            Shape::Array(span, elem, len) => {
                let elem = elem.render(leaf);
                quote_spanned!(*span=> [#elem; #len])
            }
        }
    }
}

/// Returns the associated type name if `ty` is exactly `Self::Name`.
fn as_self_assoc(ty: &Type) -> Option<&Ident> {
    let Type::Path(tp) = ty else { return None };
    if tp.qself.is_some() || tp.path.leading_colon.is_some() {
        return None;
    }
    let segs: Vec<_> = tp.path.segments.iter().collect();
    match segs[..] {
        [a, b] if a.ident == "Self" && a.arguments.is_none() && b.arguments.is_none() => {
            Some(&b.ident)
        }
        _ => None,
    }
}

/// Returns the type arguments of `ty` if it is exactly `NAME<...>` for the bare name `name`
/// and every argument is a type. `NAME` alone, with no arguments, yields an empty list.
fn as_bare_generic<'a>(ty: &'a Type, name: &str) -> Option<Vec<&'a Type>> {
    let Type::Path(tp) = ty else { return None };
    if tp.qself.is_some() || tp.path.leading_colon.is_some() || tp.path.segments.len() != 1 {
        return None;
    }
    let seg = &tp.path.segments[0];
    if seg.ident != name {
        return None;
    }
    match &seg.arguments {
        PathArguments::None => Some(vec![]),
        PathArguments::AngleBracketed(args) => args
            .args
            .iter()
            .map(|a| match a {
                GenericArgument::Type(t) => Some(t),
                _ => None,
            })
            .collect(),
        PathArguments::Parenthesized(_) => None,
    }
}

/// The last segment of `ty` if it is a path of two or more segments not starting with
/// `Self`, such as `core::option::Option<T>`.
fn as_qualified_path(ty: &Type) -> Option<&Ident> {
    let Type::Path(tp) = ty else { return None };
    if tp.qself.is_some() || tp.path.segments.len() < 2 {
        return None;
    }
    if tp.path.segments[0].ident == "Self" {
        return None;
    }
    tp.path.segments.last().map(|s| &s.ident)
}

/// Returns the span of the first `Self` token anywhere in the stream, if any.
fn find_self(ts: TokenStream) -> Option<Span> {
    for tt in ts {
        match tt {
            TokenTree::Ident(i) if i == "Self" => return Some(i.span()),
            TokenTree::Group(g) => {
                if let Some(s) = find_self(g.stream()) {
                    return Some(s);
                }
            }
            _ => {}
        }
    }
    None
}

const NESTED_HELP: &str = "`Self::Name` may only be used by value, as `&Self::Name`, `&mut Self::Name`, `Pin<&Self::Name>` or `Pin<&mut Self::Name>`, or nested by value inside `Option`, `Result`, tuples and arrays: other uses cannot be bridged across the extern symbol";
const BORROW_HELP: &str = "only `Self::Name` itself may be borrowed: references to `Option`s, `Result`s, tuples, arrays or slices containing opaque associated types are not supported; pass them by value instead";
const LIFETIME_HELP: &str =
    "explicit lifetimes on references to opaque associated types are not supported";
const PIN_HELP: &str = "`Pin` may only be used with an opaque associated type as `Pin<&Self::Name>` or `Pin<&mut Self::Name>`";

impl UnitraitInput {
    fn lookup(&self, assoc: &Ident) -> syn::Result<usize> {
        self.opaques
            .iter()
            .position(|o| o.assoc == *assoc)
            .ok_or_else(|| {
                syn::Error::new(
                    assoc.span(),
                    format!("the trait has no opaque associated type named `{assoc}`"),
                )
            })
    }

    /// Classifies one parameter or return type.
    fn classify(&self, ty: &Type) -> syn::Result<Shape> {
        // Whatever it's spelled like, a type without `Self` is passed through verbatim, so
        // `Option`, `Result` and `Pin` mean whatever they mean in scope there.
        let Some(self_span) = find_self(ty.to_token_stream()) else {
            return Ok(Shape::Plain(ty.clone()));
        };
        let leaf = |assoc: &Ident, mode: Mode| {
            Ok(Shape::Opaque {
                index: self.lookup(assoc)?,
                mode,
                span: ty.span(),
            })
        };
        if let Some(assoc) = as_self_assoc(ty) {
            return leaf(assoc, Mode::Value);
        }
        match ty {
            Type::Paren(p) => self.classify(&p.elem),
            Type::Group(g) => self.classify(&g.elem),
            Type::Reference(r) => {
                let Some(assoc) = as_self_assoc(&r.elem) else {
                    // A composite that could be passed by value gets the pointed advice;
                    // anything else the general one.
                    let composite = match &*r.elem {
                        Type::Tuple(_) | Type::Array(_) | Type::Slice(_) | Type::Paren(_) => true,
                        elem => {
                            as_bare_generic(elem, "Option").is_some()
                                || as_bare_generic(elem, "Result").is_some()
                        }
                    };
                    return Err(if composite {
                        syn::Error::new(ty.span(), BORROW_HELP)
                    } else {
                        syn::Error::new(self_span, NESTED_HELP)
                    });
                };
                if let Some(lt) = &r.lifetime {
                    return Err(syn::Error::new(lt.span(), LIFETIME_HELP));
                }
                let mode = if r.mutability.is_some() {
                    Mode::RefMut
                } else {
                    Mode::Ref
                };
                leaf(assoc, mode)
            }
            Type::Tuple(t) => {
                let elems = t
                    .elems
                    .iter()
                    .map(|e| self.classify(e))
                    .collect::<syn::Result<_>>()?;
                Ok(Shape::Tuple(ty.span(), elems))
            }
            Type::Array(a) => {
                let elem = self.classify(&a.elem)?;
                Ok(Shape::Array(ty.span(), Box::new(elem), a.len.clone()))
            }
            Type::Path(_) => {
                if let Some(args) = as_bare_generic(ty, "Pin") {
                    let [inner] = args[..] else {
                        return Err(syn::Error::new(ty.span(), PIN_HELP));
                    };
                    let Type::Reference(r) = inner else {
                        return Err(syn::Error::new(inner.span(), PIN_HELP));
                    };
                    if let Some(lt) = &r.lifetime {
                        return Err(syn::Error::new(lt.span(), LIFETIME_HELP));
                    }
                    let Some(assoc) = as_self_assoc(&r.elem) else {
                        return Err(syn::Error::new(r.elem.span(), PIN_HELP));
                    };
                    let mode = if r.mutability.is_some() {
                        Mode::PinRefMut
                    } else {
                        Mode::PinRef
                    };
                    return leaf(assoc, mode);
                }
                if let Some(args) = as_bare_generic(ty, "Option") {
                    let [inner] = args[..] else {
                        return Err(syn::Error::new(
                            ty.span(),
                            "`Option` takes exactly one type argument: around an opaque associated type it always means `core::option::Option`",
                        ));
                    };
                    let inner = self.classify(inner)?;
                    return Ok(Shape::Option(ty.span(), Box::new(inner)));
                }
                if let Some(args) = as_bare_generic(ty, "Result") {
                    let [ok, err] = args[..] else {
                        return Err(syn::Error::new(
                            ty.span(),
                            "`Result` must be written with both of its type arguments: around an opaque associated type it always means `core::result::Result`",
                        ));
                    };
                    let (ok, err) = (self.classify(ok)?, self.classify(err)?);
                    return Ok(Shape::Result(ty.span(), Box::new(ok), Box::new(err)));
                }
                if let Some(last) = as_qualified_path(ty)
                    && let Some(module) = match last.to_string().as_str() {
                        "Option" => Some("option"),
                        "Result" => Some("result"),
                        "Pin" => Some("pin"),
                        _ => None,
                    }
                {
                    return Err(syn::Error::new(
                        ty.span(),
                        format!(
                            "write `{last}` by its bare name: around an opaque associated type it always means `core::{module}::{last}`"
                        ),
                    ));
                }
                Err(syn::Error::new(self_span, NESTED_HELP))
            }
            _ => Err(syn::Error::new(self_span, NESTED_HELP)),
        }
    }

    /// The type to use for a slot in the trait declaration: `Self::Name` kept as written,
    /// `Option`, `Result` and `Pin` rewritten to their absolute paths.
    fn trait_ty(&self, shape: &Shape) -> TokenStream {
        shape.render(&|i, span| {
            let assoc = Ident::new(&self.opaques[i].assoc.to_string(), span);
            quote_spanned!(span=> Self::#assoc)
        })
    }

    /// The type to use for a slot in the dispatch methods and extern declarations
    /// (defining-crate side: opaque structs by bare name).
    fn caller_ty(&self, shape: &Shape) -> TokenStream {
        shape.render(&|i, span| {
            Ident::new(&self.opaques[i].opaque.to_string(), span).to_token_stream()
        })
    }

    /// Same as `caller_ty`, but naming the opaque structs through the user-supplied path
    /// (implementation-macro side).
    fn impl_ty(&self, shape: &Shape) -> TokenStream {
        let path = &self.path;
        shape.render(&|i, span| {
            let op = Ident::new(&self.opaques[i].opaque.to_string(), span);
            quote_spanned!(span=> #path::#op)
        })
    }
}

/// Which way a [`Bridge`] conversion goes.
#[derive(Clone, Copy)]
enum Dir {
    /// Opaque struct to the implementation's associated type: parameters.
    Unwrap,
    /// The implementation's associated type to opaque struct: return values.
    Wrap,
}

/// Generates the implementation-side shims' conversions between opaque structs and the
/// implementation's associated types, following a [`Shape`] structurally: leaves are cast
/// (by value, through the bytes; borrowed, through the pointer), and `Option`, `Result`,
/// tuples and arrays are taken apart and rebuilt around the converted leaves.
struct Bridge<'a> {
    input: &'a UnitraitInput,
    /// The implementor's type, through which the real associated types are reached.
    timpl: TokenStream,
    trait_qpath: TokenStream,
}

impl Bridge<'_> {
    /// The implementation's associated type at index `i`.
    fn real(&self, i: usize) -> TokenStream {
        let (timpl, trait_qpath) = (&self.timpl, &self.trait_qpath);
        let assoc = &self.input.opaques[i].assoc;
        quote!(<#timpl as #trait_qpath>::#assoc)
    }

    /// The opaque struct at index `i`, through the user-supplied path.
    fn qopaque(&self, i: usize) -> TokenStream {
        let path = &self.input.path;
        let op = &self.input.opaques[i].opaque;
        quote!(#path::#op)
    }

    /// An expression converting `expr`, of the shape's opaque-side type when unwrapping or
    /// its real type when wrapping, to the other.
    fn convert(&self, shape: &Shape, expr: TokenStream, dir: Dir) -> TokenStream {
        match shape {
            Shape::Plain(_) => expr,
            Shape::Opaque { index, mode, .. } => self.convert_leaf(*index, *mode, expr, dir),
            Shape::Option(_, inner) => {
                let inner = self.convert(inner, quote!(__unitrait_v), dir);
                quote! {
                    match #expr {
                        ::core::option::Option::Some(__unitrait_v) => ::core::option::Option::Some(#inner),
                        ::core::option::Option::None => ::core::option::Option::None,
                    }
                }
            }
            Shape::Result(_, ok, err) => {
                let ok = self.convert(ok, quote!(__unitrait_v), dir);
                let err = self.convert(err, quote!(__unitrait_v), dir);
                quote! {
                    match #expr {
                        ::core::result::Result::Ok(__unitrait_v) => ::core::result::Result::Ok(#ok),
                        ::core::result::Result::Err(__unitrait_v) => ::core::result::Result::Err(#err),
                    }
                }
            }
            Shape::Tuple(_, elems) => {
                let names: Vec<Ident> = (0..elems.len())
                    .map(|i| format_ident!("__unitrait_t{i}"))
                    .collect();
                let convs = elems
                    .iter()
                    .zip(&names)
                    .map(|(e, n)| self.convert(e, n.to_token_stream(), dir));
                quote! {{
                    let (#(#names,)*) = #expr;
                    (#(#convs,)*)
                }}
            }
            Shape::Array(_, elem, len) => {
                // Spelled out so that a `map` method from some trait in scope can't be
                // picked instead of the inherent one.
                let source = match dir {
                    Dir::Unwrap => self.input.impl_ty(elem),
                    Dir::Wrap => elem.render(&|i, _| self.real(i)),
                };
                let conv = self.convert(elem, quote!(__unitrait_v), dir);
                quote!(<[#source; #len]>::map(#expr, |__unitrait_v| #conv))
            }
        }
    }

    fn convert_leaf(&self, index: usize, mode: Mode, expr: TokenStream, dir: Dir) -> TokenStream {
        let (real, qopaque) = (self.real(index), self.qopaque(index));
        match (dir, mode) {
            // SAFETY (all five unwraps): opaque values always hold an initialized value of
            // the implementation's associated type (they can only be obtained from methods
            // returning one), and size and alignment are checked at compile time. The
            // by-value case takes ownership, so the opaque's own Drop must not run:
            // ManuallyDrop suppresses it and the value is moved out by reading it as the
            // real type.
            (Dir::Unwrap, Mode::Value) => quote! {{
                let __unitrait_v = ::core::mem::ManuallyDrop::new(#expr);
                unsafe {
                    (&__unitrait_v as *const ::core::mem::ManuallyDrop<#qopaque> as *const #real).read()
                }
            }},
            (Dir::Unwrap, Mode::Ref) => quote! {{
                let __unitrait_v: &#qopaque = #expr;
                unsafe { &*(__unitrait_v as *const #qopaque as *const #real) }
            }},
            (Dir::Unwrap, Mode::RefMut) => quote! {{
                let __unitrait_v: &mut #qopaque = #expr;
                unsafe { &mut *(__unitrait_v as *mut #qopaque as *mut #real) }
            }},
            // SAFETY (both pinned unwraps): as above, plus the implementation's value lives
            // at the very address the caller pinned, and the opaque struct is `Unpin` only
            // if the associated type is, so the caller could only have built this `Pin` by
            // pinning the opaque value (or with `new_unchecked`, upholding the same
            // contract). Re-pinning the value in place therefore keeps the pinning
            // guarantee the caller already granted.
            (Dir::Unwrap, Mode::PinRef) => quote! {{
                let __unitrait_v: ::core::pin::Pin<&#qopaque> = #expr;
                unsafe {
                    let __unitrait_v: &#qopaque = ::core::pin::Pin::get_ref(__unitrait_v);
                    ::core::pin::Pin::new_unchecked(&*(__unitrait_v as *const #qopaque as *const #real))
                }
            }},
            (Dir::Unwrap, Mode::PinRefMut) => quote! {{
                let __unitrait_v: ::core::pin::Pin<&mut #qopaque> = #expr;
                unsafe {
                    let __unitrait_v: &mut #qopaque = ::core::pin::Pin::get_unchecked_mut(__unitrait_v);
                    ::core::pin::Pin::new_unchecked(&mut *(__unitrait_v as *mut #qopaque as *mut #real))
                }
            }},
            // SAFETY: size and alignment are checked at compile time, so the write stays in
            // bounds and is aligned; the opaque struct's only field is uninit-tolerant
            // bytes, so `assume_init` is fine.
            (Dir::Wrap, Mode::Value) => quote! {{
                let __unitrait_v: #real = #expr;
                let mut __unitrait_out = ::core::mem::MaybeUninit::<#qopaque>::uninit();
                unsafe {
                    (__unitrait_out.as_mut_ptr() as *mut #real).write(__unitrait_v);
                    __unitrait_out.assume_init()
                }
            }},
            // A reference to the implementation's value can't be read as one to the (in
            // general larger) opaque struct. `Shape::find_borrow` rejects these in return
            // position before any code is generated.
            (Dir::Wrap, Mode::Ref | Mode::RefMut | Mode::PinRef | Mode::PinRefMut) => {
                unreachable!("borrowed opaque types are rejected in return position")
            }
        }
    }
}

fn expand(input: &UnitraitInput) -> syn::Result<TokenStream> {
    let UnitraitInput {
        trait_docs,
        vis,
        name,
        opaques,
        methods,
        dispatch_attrs,
        dispatch_vis,
        dispatch,
        mac_docs,
        mac_name,
        path,
    } = input;
    // `$t` tokens for the emitted `macro_rules!` implementation macro.
    let tvar = TokenStream::from_iter([
        TokenTree::Punct(Punct::new('$', Spacing::Alone)),
        TokenTree::Ident(Ident::new("t", Span::call_site())),
    ]);
    let trait_qpath = quote!(#path::#name);
    // The hidden trait binding the implementor's type, and the projection the shims refer to
    // it by, generic over `__B`, the type bound through the trait. See the emitted macro.
    let binding = format_ident!("__{dispatch}Implementation");
    let timpl = quote!(<__B as #path::#binding>::Type);
    let shims = quote!(__UnitraitShims::<__UnitraitTag>);

    // The opaque structs and their trait impls (defining-crate side).
    let opaque_structs = opaques.iter().map(|o| {
        let OpaqueDecl {
            docs,
            vis,
            assoc,
            opaque,
            layouts,
            bounds,
            drop_symbol,
            clone_symbol,
            ..
        } = o;
        // One impl per declared bound. The trait requires the implementation's associated
        // type to implement it, and an opaque value is just an implementation value in
        // disguise, so the two agree by construction; the bounds that aren't declared stay
        // unimplemented thanks to the marker field below.
        let marker_impls = bounds.iter().map(|b| {
            let path = b.path();
            match b {
                // SAFETY: an opaque value holds a value of the implementation's associated
                // type, which the trait bound requires to be `Send`/`Sync`.
                Marker::Send | Marker::Sync => quote!(unsafe impl #path for #opaque {}),
                // `Copy` and `Drop` are mutually exclusive bounds, so no `Drop` impl is
                // emitted for the opaque struct, and duplicating its bytes duplicates a
                // value the implementation itself declared trivially copyable.
                Marker::Copy => quote! {
                    impl #path for #opaque {}

                    impl ::core::clone::Clone for #opaque {
                        #[inline]
                        fn clone(&self) -> Self {
                            *self
                        }
                    }
                },
                // A `Clone` opaque value can only be duplicated by the implementation,
                // which knows the real type, so `clone` dispatches through a symbol like
                // any method. `clone_from` is left to its default, which goes through it.
                // The `Clone` bound requires `#[clone_symbol]`, so the `unwrap` can't fire.
                Marker::Clone => {
                    let clone_symbol = clone_symbol.as_ref().unwrap();
                    quote! {
                        impl #path for #opaque {
                            #[inline]
                            fn clone(&self) -> Self {
                                unsafe extern "Rust" {
                                    #[link_name = #clone_symbol]
                                    safe fn extern_fn(this: &#opaque) -> #opaque;
                                }
                                extern_fn(self)
                            }
                        }
                    }
                }
                _ => quote!(impl #path for #opaque {}),
            }
        });
        let drop_impl = drop_symbol.as_ref().map(|drop_symbol| {
            quote! {
                impl ::core::ops::Drop for #opaque {
                    #[inline]
                    fn drop(&mut self) {
                        unsafe extern "Rust" {
                            #[link_name = #drop_symbol]
                            safe fn extern_fn(this: &mut #opaque);
                        }
                        extern_fn(self)
                    }
                }
            }
        });
        // The struct definition for one layout. Everything else about the opaque type is
        // layout-independent and emitted once, outside the `cfg_select!` below.
        let struct_def = |align: &LitInt, padded_size: &LitInt| {
            quote! {
                #(#docs)*
                #[repr(C, align(#align))]
                #vis struct #opaque {
                    // Covers the whole struct: `padded_size` is a multiple of the alignment.
                    _data: ::core::mem::MaybeUninit<[u8; #padded_size]>,
                    // The opaque struct's auto traits must match those of the
                    // implementation's associated type, which the defining crate doesn't
                    // know. This zero-sized marker implements none of them, so they're all
                    // opted into explicitly below, guarded by the corresponding bound on the
                    // associated type.
                    _not_auto: ::core::marker::PhantomData<(
                        *mut (),
                        ::core::marker::PhantomPinned,
                        &'static mut (),
                        ::core::cell::UnsafeCell<()>,
                    )>,
                }
            }
        };
        let struct_def = match layouts.as_slice() {
            [l] if l.cfg.is_none() => struct_def(&l.align, &l.padded_size),
            // One `cfg_select!` arm per `#[opaque]`, in source order: the first predicate
            // that holds wins, and the unconditional one, if any, is last and becomes the
            // wildcard arm. This is linear in the number of attributes, unlike chaining
            // `cfg_attr`s with `not(...)` of every previous predicate.
            _ => {
                let arms = layouts.iter().map(|l| {
                    let body = struct_def(&l.align, &l.padded_size);
                    match &l.cfg {
                        Some(cfg) => quote!(#cfg => { #body }),
                        None => quote!(_ => { #body }),
                    }
                });
                // Without an unconditional `#[opaque]`, no predicate holding is an error.
                // The struct is still defined, with a dummy layout, so the error is the
                // only one reported.
                let no_match = layouts.last().is_some_and(|l| l.cfg.is_some()).then(|| {
                    let msg = format!(
                        "no `#[opaque]` attribute applies to `{assoc}`: none of its `cfg_attr` predicates hold, and it has no unconditional `#[opaque(size = N, align = M)]` fallback"
                    );
                    let error = quote_spanned!(assoc.span() => ::core::compile_error!(#msg););
                    let one = LitInt::new("1", assoc.span());
                    let zero = LitInt::new("0", assoc.span());
                    let body = struct_def(&one, &zero);
                    quote!(_ => { #error #body })
                });
                quote! {
                    ::core::cfg_select! {
                        #(#arms)*
                        #no_match
                    }
                }
            }
        };
        quote! {
            #struct_def

            #(#marker_impls)*
            #drop_impl
        }
    });

    // The trait, with items exactly as written.
    let assoc_items = opaques.iter().map(|o| {
        let OpaqueDecl {
            docs,
            assoc,
            bounds,
            ..
        } = o;
        let bounds = if bounds.is_empty() {
            quote!()
        } else {
            let paths = bounds.iter().map(|b| b.path());
            quote!(: #(#paths)+*)
        };
        quote! {
            #(#docs)*
            type #assoc #bounds;
        }
    });
    let trait_methods = methods
        .iter()
        .map(|m| {
            let Method {
                docs,
                unsafety,
                name,
                args,
                ret,
                ..
            } = m;
            let params = args
                .iter()
                .map(|(id, ty)| {
                    let tty = input.trait_ty(&input.classify(ty)?);
                    Ok(quote!(#id: #tty))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let ret_ty = match ret {
                None => quote!(),
                Some(ty) => {
                    let tty = input.trait_ty(&input.classify(ty)?);
                    quote!(-> #tty)
                }
            };
            Ok(quote! {
                #(#docs)*
                #unsafety fn #name(#(#params),*) #ret_ty;
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // The dispatch type's inherent methods (defining-crate side): one per trait method,
    // calling the global implementation through its extern symbol.
    let dispatch_methods = methods
        .iter()
        .map(|m| {
            let Method { docs, unsafety, name, symbol, args, ret } = m;
            let params = args
                .iter()
                .map(|(id, ty)| {
                    let cty = input.caller_ty(&input.classify(ty)?);
                    Ok(quote!(#id: #cty))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let ret_ty = match ret {
                None => quote!(),
                Some(ty) => {
                    let shape = input.classify(ty)?;
                    if let Some(span) = shape.find_borrow() {
                        return Err(syn::Error::new(span, "references to opaque associated types are not supported in return position"));
                    }
                    let cty = input.caller_ty(&shape);
                    quote!(-> #cty)
                }
            };
            let argids = args.iter().map(|(id, _)| id);
            let (extern_safe, call) = match unsafety {
                None => (quote!(safe), quote!(extern_fn(#(#argids),*))),
                Some(_) => (quote!(), quote!(unsafe { extern_fn(#(#argids),*) })),
            };
            Ok(quote! {
                #(#docs)*
                #[inline]
                #dispatch_vis #unsafety fn #name(#(#params),*) #ret_ty {
                    unsafe extern "Rust" {
                        #[link_name = #symbol]
                        #extern_safe fn extern_fn(#(#params),*) #ret_ty;
                    }
                    #call
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // The dispatch type's impl of the trait (defining-crate side): each opaque associated
    // type is its opaque struct, and each method forwards to the inherent method of the
    // same name. That makes the dispatch type usable wherever an implementation of the
    // trait is expected.
    let dispatch_assocs = opaques.iter().map(|o| {
        let OpaqueDecl { assoc, opaque, .. } = o;
        quote!(type #assoc = #opaque;)
    });
    let dispatch_impl_methods = methods
        .iter()
        .map(|m| {
            let Method {
                unsafety,
                name,
                args,
                ret,
                ..
            } = m;
            let params = args
                .iter()
                .map(|(id, ty)| {
                    let cty = input.caller_ty(&input.classify(ty)?);
                    Ok(quote!(#id: #cty))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let ret_ty = match ret {
                None => quote!(),
                Some(ty) => {
                    let cty = input.caller_ty(&input.classify(ty)?);
                    quote!(-> #cty)
                }
            };
            let argids = args.iter().map(|(id, _)| id);
            // Inherent methods take precedence over trait methods in path resolution, so
            // this reaches the inherent method above, not itself.
            let call = quote!(<#dispatch>::#name(#(#argids),*));
            let call = match unsafety {
                None => call,
                // SAFETY: forwarded to the caller, who called this `unsafe fn`.
                Some(_) => quote!(unsafe { #call }),
            };
            Ok(quote! {
                #[inline]
                #unsafety fn #name(#(#params),*) #ret_ty {
                    #call
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // The implementation-macro side. Everything that names the implementor's type is a
    // generic helper on `__UnitraitShims<__B>`, bounded on the binding trait: it type-checks
    // from the item bound `Type: Trait` alone, so a type that doesn't implement the trait is
    // reported exactly once, where it's bound. Exported wrappers instantiate the helpers with
    // the caller's tag. `checks` are compile-time assertions, evaluated through a const of
    // the helper; `helpers` go in its `impl`; `exports` are the extern symbols.
    let mut checks = vec![];
    let mut helpers = vec![];
    let mut exports = vec![];

    for o in opaques {
        let OpaqueDecl {
            assoc,
            opaque,
            drop_symbol,
            clone_symbol,
            ..
        } = o;
        let real = quote!(<#timpl as #trait_qpath>::#assoc);
        let qopaque = quote!(#path::#opaque);
        // Without a `Drop` bound the opaque struct has no `Drop` impl and its bytes are
        // plain `MaybeUninit`, so nothing would ever drop the implementation's value.
        // Requiring the associated type to have no drop glue at all makes that a no-op
        // rather than a leak.
        let no_drop_check = drop_symbol.is_none().then(|| quote! {
            ::core::assert!(
                !::core::mem::needs_drop::<#real>(),
                "unitrait: the implementation's associated type needs drop, but the opaque associated type has no `Drop` bound",
            );
        });
        // The declared size and alignment may depend on `cfg`s of the defining crate, which
        // can't be evaluated here, so the checks read them off the opaque struct instead.
        // Its storage covers the whole struct (see `OpaqueLayout::padded_size`), so the
        // implementation's value may use all of it.
        checks.push(quote! {
            ::core::assert!(
                ::core::mem::size_of::<#real>() <= ::core::mem::size_of::<#qopaque>(),
                "unitrait: the implementation's associated type is larger than its declared opaque size",
            );
            ::core::assert!(
                ::core::mem::align_of::<#real>() <= ::core::mem::align_of::<#qopaque>(),
                "unitrait: the implementation's associated type requires stricter alignment than its declared opaque alignment",
            );
            #no_drop_check
        });
        // Without a `Drop` bound there's no drop function to export.
        if let Some(drop_symbol) = drop_symbol {
            let drop_fn = format_ident!("__unitrait_drop_{}", assoc.to_string().to_lowercase());
            helpers.push(quote! {
                #[inline(always)]
                fn #drop_fn(this: &mut #qopaque) {
                    // SAFETY: opaque values always hold an initialized value of the
                    // implementation's associated type (they can only be obtained from
                    // methods returning one), size and alignment are checked at compile
                    // time, and this is only called from the opaque struct's `Drop` impl,
                    // so the value is never used again.
                    unsafe {
                        ::core::ptr::drop_in_place(this as *mut #qopaque as *mut #real);
                    }
                }
            });
            exports.push(quote! {
                #[unsafe(export_name = #drop_symbol)]
                fn #drop_fn(this: &mut #qopaque) {
                    #shims::#drop_fn(this)
                }
            });
        }
        // Without a `Clone` bound the opaque struct has no `Clone` impl, so there's no
        // clone function to export.
        if let Some(clone_symbol) = clone_symbol {
            let clone_fn = format_ident!("__unitrait_clone_{}", assoc.to_string().to_lowercase());
            helpers.push(quote! {
                #[inline(always)]
                fn #clone_fn(this: &#qopaque) -> #qopaque {
                    // SAFETY: opaque values always hold an initialized value of the
                    // implementation's associated type, and size and alignment are checked
                    // at compile time, so the read is in bounds and aligned, and the write
                    // stays within the opaque struct's uninit-tolerant bytes.
                    unsafe {
                        let this = &*(this as *const #qopaque as *const #real);
                        let __unitrait_cloned: #real = ::core::clone::Clone::clone(this);
                        let mut __unitrait_out = ::core::mem::MaybeUninit::<#qopaque>::uninit();
                        (__unitrait_out.as_mut_ptr() as *mut #real).write(__unitrait_cloned);
                        __unitrait_out.assume_init()
                    }
                }
            });
            exports.push(quote! {
                #[unsafe(export_name = #clone_symbol)]
                fn #clone_fn(this: &#qopaque) -> #qopaque {
                    #shims::#clone_fn(this)
                }
            });
        }
    }

    let bridge = Bridge {
        input,
        timpl: timpl.clone(),
        trait_qpath: trait_qpath.clone(),
    };
    for m in methods {
        let Method {
            unsafety,
            name,
            symbol,
            args,
            ret,
            ..
        } = m;
        let shapes = args
            .iter()
            .map(|(_, ty)| input.classify(ty))
            .collect::<syn::Result<Vec<_>>>()?;
        let params = args
            .iter()
            .zip(&shapes)
            .map(|((id, _), shape)| {
                let ity = input.impl_ty(shape);
                quote!(#id: #ity)
            })
            .collect::<Vec<_>>();
        // Convert parameters mentioning opaque types to the implementation's associated
        // types.
        let convs = args
            .iter()
            .zip(&shapes)
            .filter(|(_, shape)| !matches!(shape, Shape::Plain(_)))
            .map(|((id, _), shape)| {
                let conv = bridge.convert(shape, id.to_token_stream(), Dir::Unwrap);
                quote!(let #id = #conv;)
            });
        let argids = args.iter().map(|(id, _)| id).collect::<Vec<_>>();
        let call = quote!(<#timpl as #trait_qpath>::#name(#(#argids),*));
        let call = match unsafety {
            None => call,
            // SAFETY: forwarded to the caller through the `unsafe fn` dispatch method
            // matching this method, which carries the trait method's safety contract.
            Some(_) => quote!(unsafe { #call }),
        };
        let (ret_ty, body) = match ret {
            None => (quote!(), call),
            Some(ty) => {
                let shape = input.classify(ty)?;
                let ity = input.impl_ty(&shape);
                let body = bridge.convert(&shape, call, Dir::Wrap);
                (quote!(-> #ity), body)
            }
        };
        helpers.push(quote! {
            #[inline(always)]
            fn #name(#(#params),*) #ret_ty {
                #(#convs)*
                #body
            }
        });
        exports.push(quote! {
            #[unsafe(export_name = #symbol)]
            fn #name(#(#params),*) #ret_ty {
                #shims::#name(#(#argids),*)
            }
        });
    }

    Ok(quote! {
        #(#opaque_structs)*

        #(#trait_docs)*
        #vis trait #name {
            #(#assoc_items)*
            #(#trait_methods)*
        }

        #(#dispatch_attrs)*
        #dispatch_vis struct #dispatch;

        impl #dispatch {
            #(#dispatch_methods)*
        }

        impl #name for #dispatch {
            #(#dispatch_assocs)*
            #(#dispatch_impl_methods)*
        }

        /// Implementation detail of the implementation macro: binds the implementor's type.
        #[doc(hidden)]
        #vis trait #binding {
            type Type: #name;
        }

        #(#mac_docs)*
        #[macro_export]
        macro_rules! #mac_name {
            (#tvar:ty) => {
                const _: () = {
                    // The implementor's type is resolved here, in the caller's scope, before
                    // the glob below can shadow it: a type named like a public item of the
                    // defining module (the dispatch type, say) still means the caller's.
                    // Binding it through the trait also checks that it implements the
                    // unitrait exactly once, at the caller's token, and the shims below get
                    // that for free from the projection.
                    struct __UnitraitTag;
                    impl #path::#binding for __UnitraitTag {
                        type Type = #tvar;
                    }

                    const _: () = {
                        // Method signatures are pasted verbatim, so the types they name must
                        // resolve here as they do in the defining module.
                        #[allow(unused_imports)]
                        use #path::*;

                        struct __UnitraitShims<__B>(::core::marker::PhantomData<__B>);

                        impl<__B: #path::#binding> __UnitraitShims<__B> {
                            const CHECKS: () = {
                                #(#checks)*
                            };

                            #(#helpers)*
                        }

                        const _: () = #shims::CHECKS;

                        #(#exports)*
                    };
                };
            };
        }
    })
}
