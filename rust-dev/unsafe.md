# Unsafe

Apply every rule below to each `unsafe` block, `unsafe fn`/`trait`/`impl`, FFI boundary, `transmute`, and atomic.

- `unsafe` only with no safe equivalent; "faster" needs profiling proof plus benchmarks.
- Wrap it in a small, sound safe abstraction (ideally its own crate). A safe API that lets any input trigger UB is unsound and is a bug, even if tests pass.
- Several one-line `unsafe` blocks over one large block; each preceded by `// SAFETY:` justifying correctness. Each `unsafe fn`/`trait` documents its contract under `# Safety`.
- `transmute` and FFI need `#[repr(C)]`/`#[repr(transparent)]`; uninitialised memory goes through `MaybeUninit`, aliased mutation through `UnsafeCell`.
- Safety never depends on a safe trait impl behaving (`Eq`, `Hash`, `Ord`, `Deref` may lie or panic); require an `unsafe trait` instead.
- Cross-thread synchronisation uses `Acquire`/`Release` or stronger; `Relaxed` only for standalone counters.
