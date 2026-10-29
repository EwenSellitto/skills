---
name: rust-dev
description: Rust coding rules (Google style, Canonical discipline, cheats.rs idioms). Use when writing, refactoring, or reviewing Rust code.
---

# Rust dev

Apply every rule below to the Rust you write or review. Base layer: rustfmt, Clippy, the official Rust Style Guide, and the Rust API Guidelines; these rules win on conflict. Library code (consumed by other crates) treats API rules as binding. Application code treats them as strong defaults and favours the simple, direct version over generality. New code stays locally consistent with its surroundings.

When reviewing, report each violation with the rule it breaks. A codebase's own conventions (test framework, panic policy, lints) outrank this file.

Done means `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test` all pass clean.

## Formatting

rustfmt with `max_width = 100`, `hex_literal_case = Lower`, `format_macro_matchers = true`, `imports_granularity = Module`. Escape invisible characters in literals (`\u{200D}`). `#[rustfmt::skip]` only for deliberate alignment (matrices, aligned tables of similar lines) or generated code.

## Modules and imports

- One primary item per file, file named after it: `Foo` lives in `foo.rs`.
- Non-leaf modules hold only `mod` declarations and re-exports; implementation lives in leaf modules.
- Prefer small crates, one abstraction each. Split before adding submodules.
- Explicit imports. Glob only for `use super::*` in inline test modules, `use MyEnum::*` inside code centred on that enum, and `pub use submodule::*` in a non-leaf module.
- One module path per `use`: `use foo::bar::{Bar, Baz};`, never nested paths inside braces.
- Import blocks in order, blank line between: `std`/`core`/`alloc`, external crates, `crate`/`super`/`self`.
- Types and traits: import, then use unqualified (`Regex`). Ambiguous common names stay qualified or renamed (`io::Error`, `use std::io::Error as IoError`).
- Free functions: import the module, call qualified (`mem::swap(a, b)`), so unqualified lowercase calls always mean local.
- Traits used in one function: import inside that function body. Name clash: `use a::Trait as _;`.

## Ownership and signatures

- Unique ownership and borrows first. `Rc`/`Arc` only when the borrow-based design becomes unreasonably complex.
- `.clone()` for a real deep copy or to escape genuinely tangled borrows; borrow otherwise.
- Accept the data, not the wrapper: `&str`, `&[T]`, `&T`, `Option<&T>` (Clippy `ptr_arg`, `borrowed_box`).
- Libraries return `impl Iterator` and accept `impl IntoIterator` when they only iterate. `Vec`/slices are fine in application code or when callers index, own, or the fn exists to build a collection.
- `impl Trait` for a parameter used once; `T: Bound` generics stay simple (`AsRef<Path>`), complex bounds are a design smell.
- Bounds live on `impl` blocks, not struct definitions. Once any bound needs `where`, or the `<..>` list passes ~30 chars, move all bounds to `where`.
- Write elided lifetimes visibly: `Message<'_>`, never bare `Message`.
- Use `Self` in signatures and trait impls (`fn new(parent: &Self) -> Self`); construct with the concrete type name.
- `impl Into<T>` parameters only in library APIs where call-site ergonomics clearly outweigh signature complexity.
- Closure parameters take the loosest bound that works: `FnOnce` if called once, else `FnMut`, `Fn` only when shared or called concurrently.
- An `enum` replaces a `bool` parameter whose meaning is unclear at the call site.
- Model state so illegal states are unrepresentable: enums with data over sentinel values, newtypes (`struct Charge(f32)`) over raw primitives, guards and scopes (`thread::scope`) over manual protocols.
- Avoid global mutable state; statics duplicate silently when the crate appears in several versions.
- Named-field structs over tuples; tuple structs only as single-field newtypes.
- Newtype over type alias when the meaning differs, it needs its own API, the inner type may change, or values are easy to mix up.
- Construction: config/data structs with a sensible `Default` get pub fields; "object" types get constructors; otherwise a builder. `Foo::builder()` returns `FooBuilder` (no public `FooBuilder::new()`), setters take `self` by value, `.build()` is fallible.

## Errors and panics

- `Option` for a single obvious failure, `Result` when the caller needs context (I/O, network, parsing), `panic!` for bugs and broken invariants only, never for user input.
- Paired APIs make the panic explicit: `obtain()` panics, `try_obtain()` returns `Result`.
- Libraries define concrete error enums (`thiserror`); `anyhow`/`Box<dyn Error>` only in binaries and prototypes. Keep dependency error types out of the public API.
- Variant fields: wrapped error named `source`, free-text cause named `reason`.
- Messages: lowercase, terse, consistent verb phrasing (`"cannot open {path}"`), no context repeated by outer layers. Internal-bug panics start `internal error:`.
- Convert foreign errors with `map_err` at the call that produced them, never at the chain's end.
- Propagate with `?` (`.ok_or_else(..)?` for `Option`). A `Result<()>` fn ends `self.step()?; Ok(())`.
- `unwrap()` only in tests and on literals the programmer controls (`Regex::new("..")`). Elsewhere encode the invariant in types, or `expect("why this holds")`.
- Panicking on a poisoned lock is fine (`.lock().unwrap()`); reading through the poison is not.
- `#[track_caller]` on helpers that panic on caller misuse.

## Traits

- Implement the std trait instead of a look-alike method (`impl FromStr`, not `fn from_str`; Clippy `should_implement_trait`). `From` for always-valid conversions, `TryFrom` when it can fail.
- `#[derive]` whenever it expresses the impl, ordered `Copy`, then std traits alphabetically, then third-party alphabetically.
- Define a trait only for generic bounds, `impl Trait`/`dyn Trait` boundaries, or an extension trait. Organise code with inherent impls.
- Extension traits end in `Ext` and belong to the team owning the extended type.
- Libraries take `T: Trait`/`impl Trait` over `dyn Trait`, unless code size dominates.
- `Clone` on most types, except handles to external resources. `Copy` when the type will plausibly stay `Copy`. `Default` whenever `new()` or a natural default exists (Clippy `new_without_default`).
- `PartialEq`/`PartialOrd` by default; `Eq`/`Ord` when equality is reflexive / ordering is total. Likely map keys get `Hash + Eq`. A field order chosen for a derived `Ord` is a hand-written `Ord`.
- A `From` that is the main constructor gets a matching inherent `new()`/`from_foo()`.
- Expected API sets ship together: `len()` with `is_empty()`; custom collections implement `IntoIterator` for `C`, `&C`, `&mut C`, plus `FromIterator` and `Extend`.
- Unused params in a default trait method: `let _ = param;` as the first line.

## Naming

- Conversions: `as_` (cheap borrow), `to_` (expensive or by-value on `Copy`), `into_` (consumes `self`) (Clippy `wrong_self_convention`).
- Constructors `new()`, variants `new_with_foo()`, fallible `try_new()`, conversions `from_foo()` paired with `impl From`.
- Accessors: `foo()` returns `&T` (or `T` if `Copy`), `foo_mut()`, `set_foo(T)`, `take_foo()`, `replace_foo(T)` returning the old value, `with_foo(|v| ..)`/`with_mut_foo(|v| ..)` for values behind `Mutex`/`RefCell` instead of leaking guards.
- Pattern bindings reuse the source name: `if let Some(response) = event.response`, `State::Reading(file)`, `let Self { name, path } = self`.
- Lifetimes descriptive and named after the borrowed data (`'conn`, `'tree`), same name for the same data across an API; exceptions `'_` and at most two single-letter lifetimes tying a method's inputs to outputs.
- Generic params named for their key bound; `T`/`U` only when nearly unbounded.
- Full words, one name per concept, no type in the name (`needle`, not `needle_str`). Abbreviate only what an outsider already knows (`IpAddr`). Struggling to name it signals a bad API.

## Pattern matching

- Destructure structs exhaustively on internal code (hand-written `Ord`, `Hash`, conversions), ignoring fields with `field: _` rather than `..`, so a new field breaks the build.
- Name tuple parts by destructuring (`let (x, y) = point;`) over `.0`/`.1`, except inside a newtype's own impl.
- Unpack struct params on the first body line; keep `fn` signatures pattern-free (closures exempt).
- Deref explicitly: `.map(|x| *x)` over `.map(|&x| x)`.
- `if let`/`let else` over `is_some()` + `unwrap()`.
- `match` on a `#[non_exhaustive]` enum names every known variant, folding ignored ones into the wildcard: `Shape::Triangle | _ => {}`. For one variant, use `if let`.
- Unit arms are `=> {}`.

## Bindings and expressions

- Blocks are expressions: `let x = if c { a } else { b };`, `let x = loop { break v };`, never a declared-then-assigned `let x;`.
- Scope mutation to construction: `let list = { let mut list = Vec::new(); /* fill */ list };`.
- Take the reference at the use site: `let foo = make(); use_it(&foo);`.
- Shadow at most once per variable when the type changes; never shadow across more than one nested scope.
- Bind a block expression before calling methods on it: `let v = if c { a } else { b }; v.to_string()`.
- Early drop via a `{ }` scope over `drop(x)`. `let _ = x` drops now, `let _x = x` drops at scope end: pick deliberately for guards.
- Prefer `.collect()` over `Foo::from_iter`, `Vec::new()`/`Vec::with_capacity(n)` over `vec![]`.
- Annotate types after generic-return calls (`.parse()`, `.into()`, `.collect()`) and long iterator chains; annotate the binding over turbofish, with `_` for what the compiler knows (`Vec<_>`).
- Short combinator chains (`.iter().any(..)`) over hand loops; long, branching, or side-effecting chains become a `for` loop over a named iterator. `for` over `for_each`.
- `format!("{a}{b}{c}")` with inlined args to join three or more strings and to fill templates; `write!(s, ..)` to append to an existing `String`.
- `assert!` carries a message explaining the failure (misuse vs bug, offending value): `assert!(refund > 0, "refund must be positive, got {refund:?}")`.

## Numbers

- `as` truncates, wraps, and saturates silently. Use `u8::try_from(x)?` for narrowing, `u32::from(x)` for widening, `.round()` before float-to-int.
- Overflow panics in debug and wraps in release: pick `checked_`, `wrapping_`, or `saturating_` explicitly where input is untrusted.
- Compare floats for NaN with `is_nan()`; any `==` with NaN is false.

## Documentation

- `///` on every public item (`missing_docs`), `//!` at the top of a module/crate, `//` for implementation notes and TODOs.
- First sentence (two lines max) says when to use the item, not how it works. Refer to params by name: "the given `delta`".
- Each doc adds what the signature cannot say. A comment restating the name gets deleted.
- Sections as applicable: `# Examples`, `# Errors`, `# Panics` (every panic condition), `# Safety` (every `unsafe` fn/trait invariant).
- Doc tests prefer `no_run`/`compile_fail`/`should_panic` over `ignore`.

## Unsafe

- `unsafe` only with no safe equivalent; "faster" needs profiling proof plus benchmarks.
- Wrap it in a small, sound safe abstraction (ideally its own crate). A safe API that lets any input trigger UB is unsound and is a bug, even if tests pass.
- Several one-line `unsafe` blocks over one large block; each preceded by `// SAFETY:` justifying correctness.
- `transmute` and FFI need `#[repr(C)]`/`#[repr(transparent)]`; uninitialised memory goes through `MaybeUninit`, aliased mutation through `UnsafeCell`.
- Safety never depends on a safe trait impl behaving (`Eq`, `Hash`, `Ord`, `Deref` may lie or panic); require an `unsafe trait` instead.
- Cross-thread synchronisation uses `Acquire`/`Release` or stronger; `Relaxed` only for standalone counters.

## Async

- Never block in async code (no `std::thread::sleep`, blocking I/O, or heavy compute); offload with the runtime's blocking pool.
- Keep `!Send` values (`Rc`, `RefCell` borrows, `MutexGuard`) out of scope across `.await`.
- Any `.await` may never resume (the future can be dropped): restore invariants with a drop guard, not code after the await. Thread-locals are invalid across `.await`.

## Language features

- Stable Rust only.
- Prefer, in order: plain functions/types/traits, `macro_rules!`, external codegen, `proc_macro`.
- Operator overloading only for its mathematical meaning.
- Public enums and public-field structs shared across teams carry `#[non_exhaustive]`, unless the set is fixed (colours, bool-like states) or the struct relies on `..Default::default()` construction.
- `#[must_use]` on fns whose ignored result signals a bug or error (unless the return type already is), and on error types and lazy computations.
- `const fn` where possible, but `pub const fn` only when it will stay `const`; dropping it breaks callers.
- Items marked `#[doc(hidden)]` are private to their owner.
- `#[expect(lint)]` over `#[allow(lint)]`, with a reason.
- No code before `main()` (`ctor`), no `panic::catch_unwind`.

## Layout and ordering

- File reads top-down: public items first, helpers below. `impl Type` directly under the type; trait impls in the file of the trait or the type.
- Impl order per type: inherent, `unsafe` trait impls, then std, own, third-party trait impls.
- Struct fields: `pub`, `pub(crate)`, private.
- Helper types used by one function (serde mirrors of a remote API, test case structs) sit at the bottom of that function; remote formats never shape core types.
