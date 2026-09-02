//! Implementation detail of the [`unitrait`](https://docs.rs/unitrait) crate.
//!
//! Use the `unitrait!` macro through the `unitrait` crate; do not depend on this crate
//! directly.

use proc_macro2::{Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Expr, GenericArgument, Ident, Lit, LitInt, LitStr, PathArguments, Token,
    TraitBoundModifier, Type, TypeParamBound, Visibility, braced, parenthesized,
};

/// Define a unitrait: a trait with a single global implementation, resolved at link time.
///
/// This macro takes a trait definition and the name of an "implementation macro", and emits:
///
/// - The trait, as written.
/// - One free function per trait method, with the same name, signature and documentation,
///   which calls the same method on the global implementation. These functions are callable
///   from anywhere in the crate tree, including crates that can't see the implementation.
///   Each free function's visibility is the visibility written before `fn` on the method
///   (private if omitted, see below).
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
///         pub fn level() -> u32;
///
///         /// Returns the item associated with `token`.
///         ///
///         /// # Safety
///         ///
///         /// `token` must have been obtained from [`Item::token`].
///         #[symbol = "_frob_item"]
///         pub unsafe fn item(token: u32) -> &'static mut Item;
///     }
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
/// # fn main() { assert_eq!(level(), 42); }
/// ```
///
/// Trait methods:
///
/// - must not have a `self` parameter. If the implementation needs state, it can store it
///   in `static`s, or in a [context](#opaque-associated-types) allocated by the caller.
/// - must each have a `#[symbol = "..."]` attribute specifying the extern symbol name used
///   for that method. Symbol names must be unique across the program: prefix them
///   with your crate's name and version. Changing a method's symbol name or signature is an
///   ABI-breaking change between the defining crate and implementor crates.
/// - may be `unsafe`. The corresponding free function will be `unsafe` too.
/// - may have a visibility (e.g. `pub fn`, `pub(crate) fn`). It sets the visibility of the
///   generated free function only. Omit it to make the free function private, e.g. when
///   only the defining crate should call the unitrait, while the (more visible) trait
///   stays implementable by other crates.
///
/// The `macro NAME(path = PATH);` line sets the name of the generated implementation macro.
/// `PATH` must be the path of the module containing the `unitrait!` invocation, as seen from
/// other crates, starting with `$crate` — e.g. `$crate` if invoked at the crate root, or
/// `$crate::foo::bar` if invoked in the module `foo::bar`. It's used by the implementation
/// macro to name the trait, so the module must be publicly reachable.
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
///     pub trait Checksum {
///         /// Opaque storage for the implementation's checksum state.
///         #[opaque(size = 16, align = 8)]
///         #[symbol = "_cksum_context_drop"]
///         pub type Context;
///
///         /// Returns a fresh checksum state.
///         #[symbol = "_cksum_init"]
///         pub fn cksum_init() -> Self::Context;
///
///         /// Absorbs `data` into the checksum state.
///         #[symbol = "_cksum_update"]
///         pub fn cksum_update(ctx: &mut Self::Context, data: &[u8]);
///
///         /// Returns the checksum of the absorbed data.
///         #[symbol = "_cksum_finish"]
///         pub fn cksum_finish(ctx: &Self::Context) -> u32;
///     }
///
///     /// Set the global checksum implementation.
///     macro checksum_impl(path = $crate);
/// }
/// # struct Fletcher;
/// # impl Checksum for Fletcher {
/// #     type Context = (u32, u32);
/// #     fn cksum_init() -> (u32, u32) { (0, 0) }
/// #     fn cksum_update(ctx: &mut (u32, u32), data: &[u8]) {
/// #         for &b in data { ctx.0 = (ctx.0 + b as u32) % 65535; ctx.1 = (ctx.1 + ctx.0) % 65535; }
/// #     }
/// #     fn cksum_finish(ctx: &(u32, u32)) -> u32 { ctx.1 << 16 | ctx.0 }
/// # }
/// # checksum_impl!(Fletcher);
/// # fn main() {
/// #     let mut ctx = cksum_init();
/// #     cksum_update(&mut ctx, b"hello");
/// #     assert_ne!(cksum_finish(&ctx), 0);
/// # }
/// ```
///
/// Each opaque associated type declaration must be of the form `#[opaque(size = N, align = M)] #[symbol = "..."] [vis] type Name[: Bounds];`, where `N` and
/// `M` are integer literals and `Bounds` is described under [marker
/// bounds](#marker-bounds-on-opaque-types). It emits:
///
/// - `type Name;` in the trait, with the declared bounds. The implementation sets it to a
///   type of its choosing, which must have size at most `N` and alignment at most `M`; the
///   implementation macro verifies both at compile time.
/// - The opaque struct, named by concatenating the trait name and the associated type name
///   (`ChecksumContext` above), laid out as `MaybeUninit<[u8; N]>` with alignment `M`. Its
///   visibility is the one written on the `type` declaration (private if omitted, like free
///   functions); note the implementation macro and the free functions name it, so it must
///   be visible wherever the trait is implemented or the type is used.
///
/// An opaque struct value always holds an initialized value of the implementation's
/// (unknown to the caller) associated type: the only way to obtain one is through a method
/// that returns it, and its `Drop` impl drops the implementation's value in place, through
/// the extern symbol given by the `#[symbol = ...]` attribute on the declaration. This is
/// why methods taking opaque types are safe.
///
/// Methods may use `Self::Name` at the *top level* of any parameter and of the return type,
/// in these forms:
///
/// - `Self::Name` (by value): the free function takes or returns the opaque struct by
///   value. A by-value parameter consumes the value, and it is dropped (or kept) by the
///   implementation.
/// - `&Self::Name` and `&mut Self::Name` (parameters only): the free function takes a
///   reference to the opaque struct.
/// - `Pin<&Self::Name>` and `Pin<&mut Self::Name>` (parameters only): the free function
///   takes a pinned reference to the opaque struct, and the implementation receives a
///   pinned reference to its own value, at the address the caller pinned. Write `Pin` by
///   its bare name; it always means [`core::pin::Pin`], whatever `Pin` happens to be in
///   scope, and is rewritten to its absolute path everywhere it's emitted.
///
/// Except for that rewriting of `Pin`, they appear in the trait exactly as written, so
/// implementations are ordinary safe code. Nested uses (`Option<Self::Name>`,
/// `&[Self::Name]`, ...) are rejected at compile time: they cannot be bridged across the
/// extern symbol, since the caller-side and implementation-side types are not
/// layout-compatible beyond the top level.
///
/// # Marker bounds on opaque types
///
/// The caller can't see the implementation's associated type, so the opaque struct can't
/// derive its auto traits from it. Instead, the opaque struct implements *no* auto trait by
/// default, and each one is opted into by declaring it as a bound on the associated type:
///
/// ```
/// unitrait::unitrait! {
///     pub trait Session {
///         /// Can be moved between threads, but not shared, and must not be moved once
///         /// it has been pinned.
///         #[opaque(size = 64, align = 8)]
///         #[symbol = "_session_state_drop"]
///         pub type State: Send;
///
///         /// A plain copyable handle: no drop glue, and duplicating it is free.
///         #[opaque(size = 4, align = 4)]
///         pub type Id: Copy + Send + Sync;
///
///         #[symbol = "_session_open"]
///         pub fn session_open() -> Self::State;
///
///         #[symbol = "_session_id"]
///         pub fn session_id(state: &Self::State) -> Self::Id;
///     }
///
///     macro session_impl(path = $crate);
/// }
/// # struct MySession;
/// # impl Session for MySession {
/// #     type State = u64;
/// #     type Id = u32;
/// #     fn session_open() -> u64 { 7 }
/// #     fn session_id(state: &u64) -> u32 { *state as u32 }
/// # }
/// # session_impl!(MySession);
/// # fn main() { let s = session_open(); assert_eq!(core::mem::size_of_val(&session_id(&s)), 4); }
/// ```
///
/// The supported bounds are `Send`, `Sync`, `Unpin`, `UnwindSafe`, `RefUnwindSafe` and
/// `Copy`. Each one is emitted both as a bound on the associated type — so the compiler
/// rejects an implementation whose type doesn't implement it — and as an `impl` on the
/// opaque struct, which is therefore never more permissive than the implementation's own
/// type. They must be written by their bare names; they always mean the `core` traits, so a
/// trait of the same name in scope where `unitrait!` is invoked changes nothing, and no
/// other bound (including lifetimes and `?Sized`) is accepted.
///
/// `Copy` additionally means the opaque struct has no drop glue: no `Drop` impl is emitted
/// for it, and the declaration must not carry a `#[symbol = "..."]` attribute, since there
/// is no drop function to export. `Clone` is implemented alongside `Copy`.
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
/// When implementing from within the defining crate itself, the implementation macro must be
/// invoked by its bare name (textually scoped, like all `macro_rules!` macros), not through a
/// `crate::` path: `#[macro_use]` on the defining module can be used to extend its textual
/// scope to the rest of the crate.
///
/// # Name resolution in method signatures
///
/// The method signatures are pasted verbatim both into the defining crate (for the trait and
/// free functions) and into implementor crates (by the implementation macro). Therefore, the
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
/// Because of that `use PATH::*;` glob, if the type passed to the implementation macro has the
/// same name as a public item of the defining module (e.g. a driver struct named exactly like
/// the trait), the bare name is ambiguous inside the macro expansion. Qualify it to disambiguate:
/// `some_driver_impl!(self::Driver)`.
///
/// # Linkage details
///
/// Calls from the free functions to the implementation are done via `extern "Rust"` functions.
///
/// For each method, the free function calls `extern "Rust" { fn "SYMBOL"(...); }`, and the
/// implementation macro exports `#[export_name = "SYMBOL"] fn(...)` which forwards to the
/// trait implementation. The linker resolves the former to the latter. If no crate in the tree
/// invokes the implementation macro, linking fails with an "undefined symbol" error; if more
/// than one does, it fails with a "duplicate symbol" error.
///
/// For opaque associated types, both sides of the symbol pass the opaque struct; the exported
/// functions cast it to the implementation's associated type (sound thanks to the size/align
/// checks and the always-initialized invariant). The implementation macro additionally exports
/// each opaque type's drop symbol, whose function drops the implementation's value in place;
/// the opaque struct's `Drop` impl calls it.
///
/// Because the contract between the two sides is just the symbol name and signature, crates
/// may define and implement the unitrait through *different versions* of the defining crate
/// (as happens during major-version transitions of the defining crate) and still link
/// correctly, as long as the symbol names and signatures match. For opaque types, the size
/// and alignment are part of that ABI contract too: shrinking either is ABI-breaking, and
/// so is changing the declared marker bounds, since the two sides would then disagree on
/// what the opaque struct implements.
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
    mac_docs: Vec<Attribute>,
    mac_name: Ident,
    path: TokenStream,
}

struct OpaqueDecl {
    docs: Vec<Attribute>,
    vis: Visibility,
    /// The associated type's name, e.g. `Context`.
    assoc: Ident,
    /// The generated opaque struct's name: trait name + associated type name.
    opaque: Ident,
    size: LitInt,
    align: LitInt,
    /// The marker traits declared as bounds on the associated type.
    bounds: Vec<Marker>,
    /// The symbol of the function dropping the value in place. `None` for `Copy` opaque
    /// types, which have no drop glue.
    drop_symbol: Option<LitStr>,
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
}

const MARKERS: &[(&str, Marker)] = &[
    ("Send", Marker::Send),
    ("Sync", Marker::Sync),
    ("Unpin", Marker::Unpin),
    ("UnwindSafe", Marker::UnwindSafe),
    ("RefUnwindSafe", Marker::RefUnwindSafe),
    ("Copy", Marker::Copy),
];

const BOUND_HELP: &str = "only the marker traits `Send`, `Sync`, `Unpin`, `UnwindSafe`, `RefUnwindSafe` and `Copy` are allowed as bounds on an opaque associated type, written by their bare names";

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
        }
    }
}

/// Parses the optional `: Send + Sync` bounds of an opaque associated type declaration.
fn parse_bounds(input: ParseStream) -> syn::Result<Vec<Marker>> {
    let mut bounds = vec![];
    if !input.peek(Token![:]) {
        return Ok(bounds);
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
        let Some(marker) = Marker::from_ident(ident) else {
            return Err(syn::Error::new(ident.span(), BOUND_HELP));
        };
        if bounds.contains(&marker) {
            return Err(syn::Error::new(
                ident.span(),
                format!("duplicate `{}` bound", marker.name()),
            ));
        }
        bounds.push(marker);
    }
    Ok(bounds)
}

struct Method {
    docs: Vec<Attribute>,
    vis: Visibility,
    unsafety: Option<Token![unsafe]>,
    name: Ident,
    symbol: LitStr,
    args: Vec<(Ident, Type)>,
    ret: Option<Type>,
}

type SplitAttrs = (Vec<Attribute>, Option<LitStr>, Option<(LitInt, LitInt)>);

/// Splits attributes into doc comments, an optional `#[symbol = "..."]`, and an optional
/// `#[opaque(size = N, align = M)]`.
fn split_attrs(attrs: Vec<Attribute>) -> syn::Result<SplitAttrs> {
    let mut docs = vec![];
    let mut symbol = None;
    let mut opaque = None;
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
            let syn::Meta::NameValue(nv) = &attr.meta else {
                return Err(syn::Error::new(
                    attr.span(),
                    "expected `#[symbol = \"...\"]`",
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
            symbol = Some(s.clone());
        } else if attr.path().is_ident("opaque") {
            if opaque.is_some() {
                return Err(syn::Error::new(
                    attr.span(),
                    "duplicate `#[opaque]` attribute",
                ));
            }
            let mut size = None;
            let mut align = None;
            attr.parse_nested_meta(|meta| {
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
                    attr.span(),
                    "expected `#[opaque(size = N, align = M)]`",
                ));
            };
            opaque = Some((size, align));
        } else {
            return Err(syn::Error::new(
                attr.span(),
                "unexpected attribute; expected doc comments, `#[symbol = \"...\"]` or `#[opaque(size = N, align = M)]`",
            ));
        }
    }
    Ok((docs, symbol, opaque))
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
        let trait_docs = only_docs(input.call(Attribute::parse_outer)?)?;
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
                let bounds = parse_bounds(&content)?;
                if content.peek(Token![=]) {
                    return Err(content.error(format!(
                        "the opaque type's name is derived automatically as `{name}{assoc}` (trait name + associated type name); remove the `= ...`"
                    )));
                }
                content.parse::<Token![;]>()?;
                let (docs, symbol, opaque) = split_attrs(attrs)?;
                let drop_symbol = match (symbol, bounds.contains(&Marker::Copy)) {
                    (symbol @ Some(_), false) => symbol,
                    (None, true) => None,
                    (None, false) => {
                        return Err(syn::Error::new(
                            assoc.span(),
                            "opaque associated types require a `#[symbol = \"...\"]` attribute naming the extern symbol for dropping the value in place",
                        ));
                    }
                    (Some(symbol), true) => {
                        return Err(syn::Error::new(
                            symbol.span(),
                            "`Copy` opaque associated types have no drop glue, so they must not have a `#[symbol = \"...\"]` attribute",
                        ));
                    }
                };
                let Some((size, align)) = opaque else {
                    return Err(syn::Error::new(
                        assoc.span(),
                        "opaque associated types require an `#[opaque(size = N, align = M)]` attribute",
                    ));
                };
                if opaques.iter().any(|o| o.assoc == assoc) {
                    return Err(syn::Error::new(assoc.span(), "duplicate associated type"));
                }
                let opaque = format_ident!("{name}{assoc}", span = assoc.span());
                opaques.push(OpaqueDecl {
                    docs,
                    vis: ivis,
                    assoc,
                    opaque,
                    size,
                    align,
                    bounds,
                    drop_symbol,
                });
            } else {
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
                let (docs, symbol, opaque) = split_attrs(attrs)?;
                if opaque.is_some() {
                    return Err(syn::Error::new(
                        fname.span(),
                        "`#[opaque]` is only allowed on associated type declarations",
                    ));
                }
                let Some(symbol) = symbol else {
                    return Err(syn::Error::new(
                        fname.span(),
                        "unitrait methods require a `#[symbol = \"...\"]` attribute",
                    ));
                };
                methods.push(Method {
                    docs,
                    vis: ivis,
                    unsafety,
                    name: fname,
                    symbol,
                    args,
                    ret,
                });
            }
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
            mac_docs,
            mac_name,
            path,
        })
    }
}

/// How a method uses a type slot (one parameter, or the return type).
enum Slot {
    /// An ordinary type, containing no `Self`.
    Plain,
    /// `Self::Name` by value.
    Value(usize),
    /// `&Self::Name`.
    Ref(usize),
    /// `&mut Self::Name`.
    RefMut(usize),
    /// `Pin<&Self::Name>`.
    PinRef(usize),
    /// `Pin<&mut Self::Name>`.
    PinRefMut(usize),
}

impl Slot {
    /// Whether the slot borrows an opaque value, and therefore can't be returned.
    fn is_borrow(&self) -> bool {
        matches!(
            self,
            Slot::Ref(_) | Slot::RefMut(_) | Slot::PinRef(_) | Slot::PinRefMut(_)
        )
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

/// Returns the single generic argument of `ty` if it is exactly `Pin<T>` for some `T`
/// mentioning `Self`.
///
/// Only the bare name `Pin` is recognized, and it's always rewritten to
/// `::core::pin::Pin`, so a `Pin` shadowing `core`'s in either the defining or the
/// implementing crate can't change what the generated code means.
fn as_pin_self(ty: &Type) -> Option<&Type> {
    let Type::Path(tp) = ty else { return None };
    if tp.qself.is_some() || tp.path.leading_colon.is_some() || tp.path.segments.len() != 1 {
        return None;
    }
    let seg = &tp.path.segments[0];
    if seg.ident != "Pin" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    let GenericArgument::Type(inner) = &args.args[0] else {
        return None;
    };
    find_self(inner.to_token_stream())?;
    Some(inner)
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

    fn classify(&self, ty: &Type) -> syn::Result<Slot> {
        if let Some(assoc) = as_self_assoc(ty) {
            return Ok(Slot::Value(self.lookup(assoc)?));
        }
        if let Type::Reference(r) = ty
            && let Some(assoc) = as_self_assoc(&r.elem)
        {
            if let Some(lt) = &r.lifetime {
                return Err(syn::Error::new(
                    lt.span(),
                    "explicit lifetimes on references to opaque associated types are not supported",
                ));
            }
            let i = self.lookup(assoc)?;
            return Ok(if r.mutability.is_some() {
                Slot::RefMut(i)
            } else {
                Slot::Ref(i)
            });
        }
        if let Some(inner) = as_pin_self(ty) {
            let Type::Reference(r) = inner else {
                return Err(syn::Error::new(
                    inner.span(),
                    "`Pin` may only be used with an opaque associated type as `Pin<&Self::Name>` or `Pin<&mut Self::Name>`",
                ));
            };
            if let Some(lt) = &r.lifetime {
                return Err(syn::Error::new(
                    lt.span(),
                    "explicit lifetimes on references to opaque associated types are not supported",
                ));
            }
            let Some(assoc) = as_self_assoc(&r.elem) else {
                return Err(syn::Error::new(
                    r.elem.span(),
                    "`Pin` may only be used with an opaque associated type as `Pin<&Self::Name>` or `Pin<&mut Self::Name>`",
                ));
            };
            let i = self.lookup(assoc)?;
            return Ok(if r.mutability.is_some() {
                Slot::PinRefMut(i)
            } else {
                Slot::PinRef(i)
            });
        }
        if let Some(span) = find_self(ty.to_token_stream()) {
            return Err(syn::Error::new(
                span,
                "`Self` may only appear as `Self::Name`, `&Self::Name`, `&mut Self::Name`, `Pin<&Self::Name>` or `Pin<&mut Self::Name>` at the top level of a parameter or return type: nested uses cannot be bridged across the extern symbol",
            ));
        }
        Ok(Slot::Plain)
    }

    /// The type to use for a slot in the trait declaration, with `Self::Name` kept as
    /// written and `Pin` rewritten to its absolute path.
    fn trait_ty(&self, ty: &Type, slot: &Slot) -> TokenStream {
        match slot {
            Slot::PinRef(i) => {
                let assoc = &self.opaques[*i].assoc;
                quote!(::core::pin::Pin<&Self::#assoc>)
            }
            Slot::PinRefMut(i) => {
                let assoc = &self.opaques[*i].assoc;
                quote!(::core::pin::Pin<&mut Self::#assoc>)
            }
            _ => ty.to_token_stream(),
        }
    }

    /// The type to use for a slot in the free functions and extern declarations
    /// (defining-crate side: opaque structs by bare name).
    fn caller_ty(&self, ty: &Type, slot: &Slot) -> TokenStream {
        match slot {
            Slot::Plain => ty.to_token_stream(),
            Slot::Value(i) => self.opaques[*i].opaque.to_token_stream(),
            Slot::Ref(i) => {
                let op = &self.opaques[*i].opaque;
                quote!(&#op)
            }
            Slot::RefMut(i) => {
                let op = &self.opaques[*i].opaque;
                quote!(&mut #op)
            }
            Slot::PinRef(i) => {
                let op = &self.opaques[*i].opaque;
                quote!(::core::pin::Pin<&#op>)
            }
            Slot::PinRefMut(i) => {
                let op = &self.opaques[*i].opaque;
                quote!(::core::pin::Pin<&mut #op>)
            }
        }
    }

    /// Same as `caller_ty`, but naming the opaque structs through the user-supplied path
    /// (implementation-macro side).
    fn impl_ty(&self, ty: &Type, slot: &Slot) -> TokenStream {
        let path = &self.path;
        match slot {
            Slot::Plain => ty.to_token_stream(),
            Slot::Value(i) => {
                let op = &self.opaques[*i].opaque;
                quote!(#path::#op)
            }
            Slot::Ref(i) => {
                let op = &self.opaques[*i].opaque;
                quote!(&#path::#op)
            }
            Slot::RefMut(i) => {
                let op = &self.opaques[*i].opaque;
                quote!(&mut #path::#op)
            }
            Slot::PinRef(i) => {
                let op = &self.opaques[*i].opaque;
                quote!(::core::pin::Pin<&#path::#op>)
            }
            Slot::PinRefMut(i) => {
                let op = &self.opaques[*i].opaque;
                quote!(::core::pin::Pin<&mut #path::#op>)
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

    // The opaque structs and their Drop impls (defining-crate side).
    let opaque_structs = opaques.iter().map(|o| {
        let OpaqueDecl {
            docs,
            vis,
            opaque,
            size,
            align,
            bounds,
            drop_symbol,
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
                // A `Copy` associated type has no drop glue, so no `Drop` impl is emitted
                // for the opaque struct and duplicating its bytes duplicates a value the
                // implementation itself declared trivially copyable.
                Marker::Copy => quote! {
                    impl #path for #opaque {}

                    impl ::core::clone::Clone for #opaque {
                        #[inline]
                        fn clone(&self) -> Self {
                            *self
                        }
                    }
                },
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
        quote! {
            #(#docs)*
            #[repr(C, align(#align))]
            #vis struct #opaque {
                _data: ::core::mem::MaybeUninit<[u8; #size]>,
                // The opaque struct's auto traits must match those of the implementation's
                // associated type, which the defining crate doesn't know. This zero-sized
                // marker implements none of them, so they're all opted into explicitly
                // above, guarded by the corresponding bound on the associated type.
                _not_auto: ::core::marker::PhantomData<(
                    *mut (),
                    ::core::marker::PhantomPinned,
                    &'static mut (),
                    ::core::cell::UnsafeCell<()>,
                )>,
            }

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
                    let tty = input.trait_ty(ty, &input.classify(ty)?);
                    Ok(quote!(#id: #tty))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let ret_ty = match ret {
                None => quote!(),
                Some(ty) => {
                    let tty = input.trait_ty(ty, &input.classify(ty)?);
                    quote!(-> #tty)
                }
            };
            Ok(quote! {
                #(#docs)*
                #unsafety fn #name(#(#params),*) #ret_ty;
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // The free functions (defining-crate side).
    let free_fns = methods
        .iter()
        .map(|m| {
            let Method { docs, vis, unsafety, name, symbol, args, ret } = m;
            let params = args
                .iter()
                .map(|(id, ty)| {
                    let cty = input.caller_ty(ty, &input.classify(ty)?);
                    Ok(quote!(#id: #cty))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let ret_ty = match ret {
                None => quote!(),
                Some(ty) => {
                    let slot = input.classify(ty)?;
                    if slot.is_borrow() {
                        return Err(syn::Error::new(ty.span(), "references to opaque associated types are not supported in return position"));
                    }
                    let cty = input.caller_ty(ty, &slot);
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
                #vis #unsafety fn #name(#(#params),*) #ret_ty {
                    unsafe extern "Rust" {
                        #[link_name = #symbol]
                        #extern_safe fn extern_fn(#(#params),*) #ret_ty;
                    }
                    #call
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // Compile-time checks and drop shims for the opaque types (implementation-macro side).
    let opaque_shims = opaques.iter().map(|o| {
        let OpaqueDecl { assoc, opaque, size, align, drop_symbol, .. } = o;
        let real = quote!(<#tvar as #trait_qpath>::#assoc);
        let qopaque = quote!(#path::#opaque);
        let drop_fn = format_ident!("__unitrait_drop_{}", assoc.to_string().to_lowercase());
        // `Copy` opaque types have no `Drop` impl, so there's nothing to export.
        let drop_shim = drop_symbol.as_ref().map(|drop_symbol| {
            quote! {
                #[unsafe(export_name = #drop_symbol)]
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
            }
        });
        quote! {
            const _: () = {
                ::core::assert!(
                    ::core::mem::size_of::<#real>() <= #size,
                    "unitrait: the implementation's associated type is larger than its declared opaque size",
                );
                ::core::assert!(
                    ::core::mem::align_of::<#real>() <= #align,
                    "unitrait: the implementation's associated type requires stricter alignment than its declared opaque alignment",
                );
            };

            #drop_shim
        }
    });

    // The method shims (implementation-macro side).
    let method_shims = methods
        .iter()
        .map(|m| {
            let Method { unsafety, name, symbol, args, ret, .. } = m;
            let slots = args.iter().map(|(_, ty)| input.classify(ty)).collect::<syn::Result<Vec<_>>>()?;
            let params = args
                .iter()
                .zip(&slots)
                .map(|((id, ty), slot)| {
                    let ity = input.impl_ty(ty, slot);
                    quote!(#id: #ity)
                })
                .collect::<Vec<_>>();
            // Convert opaque parameters to the implementation's associated type.
            let convs = args.iter().zip(&slots).filter_map(|((id, _), slot)| {
                let real = |i: &usize| {
                    let assoc = &input.opaques[*i].assoc;
                    quote!(<#tvar as #trait_qpath>::#assoc)
                };
                let qopaque = |i: &usize| {
                    let op = &input.opaques[*i].opaque;
                    quote!(#path::#op)
                };
                match slot {
                    Slot::Plain => None,
                    // SAFETY (all three): opaque values always hold an initialized value of
                    // the implementation's associated type, and size and alignment are
                    // checked at compile time. The by-value case takes ownership, so the
                    // opaque's own Drop must not run: ManuallyDrop suppresses it and the
                    // value is moved out by reading it as the real type.
                    Slot::Value(i) => {
                        let (real, qopaque) = (real(i), qopaque(i));
                        Some(quote! {
                            let #id = ::core::mem::ManuallyDrop::new(#id);
                            let #id: #real = unsafe {
                                (&#id as *const ::core::mem::ManuallyDrop<#qopaque> as *const #real).read()
                            };
                        })
                    }
                    Slot::Ref(i) => {
                        let (real, qopaque) = (real(i), qopaque(i));
                        Some(quote! {
                            let #id = unsafe { &*(#id as *const #qopaque as *const #real) };
                        })
                    }
                    Slot::RefMut(i) => {
                        let (real, qopaque) = (real(i), qopaque(i));
                        Some(quote! {
                            let #id = unsafe { &mut *(#id as *mut #qopaque as *mut #real) };
                        })
                    }
                    // SAFETY (both): as above, plus the implementation's value lives at the
                    // very address the caller pinned, and the opaque struct is `Unpin` only
                    // if the associated type is, so the caller could only have built this
                    // `Pin` by pinning the opaque value (or with `new_unchecked`, upholding
                    // the same contract). Re-pinning the value in place therefore keeps the
                    // pinning guarantee the caller already granted.
                    Slot::PinRef(i) => {
                        let (real, qopaque) = (real(i), qopaque(i));
                        Some(quote! {
                            let #id = unsafe {
                                let #id: &#qopaque = ::core::pin::Pin::get_ref(#id);
                                ::core::pin::Pin::new_unchecked(&*(#id as *const #qopaque as *const #real))
                            };
                        })
                    }
                    Slot::PinRefMut(i) => {
                        let (real, qopaque) = (real(i), qopaque(i));
                        Some(quote! {
                            let #id = unsafe {
                                let #id: &mut #qopaque = ::core::pin::Pin::get_unchecked_mut(#id);
                                ::core::pin::Pin::new_unchecked(&mut *(#id as *mut #qopaque as *mut #real))
                            };
                        })
                    }
                }
            });
            let argids = args.iter().map(|(id, _)| id);
            let call = quote!(<#tvar as #trait_qpath>::#name(#(#argids),*));
            let call = match unsafety {
                None => call,
                // SAFETY: forwarded to the caller through the `unsafe fn` free function
                // matching this method, which carries the trait method's safety contract.
                Some(_) => quote!(unsafe { #call }),
            };
            let (ret_ty, body) = match ret {
                None => (quote!(), call),
                Some(ty) => {
                    let slot = input.classify(ty)?;
                    let ity = input.impl_ty(ty, &slot);
                    let body = match &slot {
                        Slot::Value(i) => {
                            let assoc = &input.opaques[*i].assoc;
                            let real = quote!(<#tvar as #trait_qpath>::#assoc);
                            let qopaque = {
                                let op = &input.opaques[*i].opaque;
                                quote!(#path::#op)
                            };
                            // SAFETY: size and alignment are checked at compile time, so the
                            // write stays in bounds and is aligned; the opaque struct's only
                            // field is uninit-tolerant bytes, so `assume_init` is fine.
                            quote! {
                                let __unitrait_ret: #real = #call;
                                let mut __unitrait_out = ::core::mem::MaybeUninit::<#qopaque>::uninit();
                                unsafe {
                                    (__unitrait_out.as_mut_ptr() as *mut #real).write(__unitrait_ret);
                                    __unitrait_out.assume_init()
                                }
                            }
                        }
                        _ => call,
                    };
                    (quote!(-> #ity), body)
                }
            };
            Ok(quote! {
                #[unsafe(export_name = #symbol)]
                fn #name(#(#params),*) #ret_ty {
                    #(#convs)*
                    #body
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        #(#opaque_structs)*

        #(#trait_docs)*
        #vis trait #name {
            #(#assoc_items)*
            #(#trait_methods)*
        }

        #(#free_fns)*

        #(#mac_docs)*
        #[macro_export]
        macro_rules! #mac_name {
            (#tvar:ty) => {
                const _: () = {
                    #[allow(unused_imports)]
                    use #path::*;

                    #(#opaque_shims)*
                    #(#method_shims)*
                };
            };
        }
    })
}
