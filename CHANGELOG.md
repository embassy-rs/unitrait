# Changelog for unitrait

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## Unreleased - ReleaseDate

- Added marker trait bounds on opaque associated types: `type Name: Send + Sync;`. The
  supported bounds are `Send`, `Sync`, `Unpin`, `UnwindSafe`, `RefUnwindSafe` and `Copy`.
- **Breaking, soundness fix**: opaque types no longer implement `Send`, `Sync`, `Unpin`,
  `UnwindSafe` or `RefUnwindSafe` unless declared as a bound on the associated type.
  Previously they implemented all of them unconditionally, regardless of what the
  implementation's associated type implemented, which let safe code send a `!Send`
  implementation value across threads.
- Added support for `Pin<&Self::Name>` and `Pin<&mut Self::Name>` method parameters.
- Added the `Drop` bound on opaque associated types: `type Name: Send + Drop;`. It declares
  that the opaque type has drop glue, and requires a `#[drop_symbol = "..."]` attribute
  naming the symbol to drop through. It's mutually exclusive with `Copy`.
- Added the `Clone` bound on opaque associated types. It requires a
  `#[clone_symbol = "..."]` attribute, and `clone` dispatches through that symbol, since only
  the implementation can duplicate a value of a type the caller can't see. `clone_from` is
  left at its default, so it goes through `clone` too. It's mutually exclusive with `Copy`,
  which already provides `Clone`.
- **Breaking**: an opaque associated type without a `Drop` bound now has no `Drop` impl, and
  the implementation macro checks at compile time that the implementation's associated type
  has no drop glue. Previously every non-`Copy` opaque type had drop glue.
- **Breaking**: opaque associated types now name their drop symbol with
  `#[drop_symbol = "..."]` instead of `#[symbol = "..."]`, which is now rejected on them.
  `#[symbol = "..."]` keeps its meaning on methods, and is now the only place it's allowed.

## 1.1.0 - 2026-08-26

- Added support for associated types.
- Reimplemented as a proc macro.

## 1.0.0 - 2026-08-12

Initial release
