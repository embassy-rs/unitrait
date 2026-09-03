# Changelog for unitrait

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## 2.0.0 - ReleaseDate

- **Breaking:** calls go through a *dispatch type* instead of free functions. A `struct NAME;`
  line between the trait and the `macro` line names it; it gets one inherent method per trait
  method (`NAME::method(...)`) and implements the trait itself. Several unitraits in one
  module may therefore share method names.
  - Trait methods must no longer have a visibility: the dispatch type's visibility applies
    to all of its methods.
  - Opaque structs are named after the dispatch type, not the trait.
- Added opaque associated types: `#[opaque(size = N, align = M)] type Name;` declares a type
  the caller sees as opaque bytes of a fixed maximum size and alignment, while the
  implementation picks the real type.
  - Methods may take one as `Self::Name`, `&Self::Name`, `&mut Self::Name`, `Pin<&Self::Name>` or `Pin<&mut Self::Name>`.
  - Methods may return one by value only.
  - The implementation macro checks the size and alignment at compile time.
  - `Send`, `Sync`, `Unpin`, `UnwindSafe`, `RefUnwindSafe` and `Copy` may be declared as bounds.
  - A `Drop` bound gives the opaque type drop glue, dropped through its `#[drop_symbol = "..."]`.
  - A `Clone` bound makes it cloneable through its `#[clone_symbol = "..."]`.
- Added `#[symbol_prefix = "..."]` on the trait, deriving every symbol name and making `#[symbol]`, `#[drop_symbol]` and `#[clone_symbol]` optional overrides.
- Reimplemented as a proc macro.

## 1.1.0 - 2026-08-26 **YANKED**

Yanked: its opaque associated types were unsound, since they implemented every auto trait
regardless of what the implementation's associated type implemented, which let safe code
send a `!Send` implementation value across threads. Use 1.2.0, which contains everything
this release added.

## 1.0.0 - 2026-08-12

Initial release
