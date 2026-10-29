# Async

Apply every rule below to each `async` fn/block and every `.await`.

- Never block in async code (no `std::thread::sleep`, blocking I/O, or heavy compute); offload with the runtime's blocking pool.
- Keep `!Send` values (`Rc`, `RefCell` borrows, `MutexGuard`) out of scope across `.await`.
- Any `.await` may never resume (the future can be dropped): restore invariants with a drop guard, not code after the await. Thread-locals are invalid across `.await`.
