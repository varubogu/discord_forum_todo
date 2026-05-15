# ADR 0002 — Static dispatch by default

- Status: Accepted
- Date: 2026-05-14

## Context

Rust offers two ways to abstract over an interface:

- **Static dispatch** — generics with trait bounds (`fn foo<R: Repo>(r: R)`). Each instantiation is a separate monomorphised copy. No vtable, no allocation, inlinable.
- **Dynamic dispatch** — trait objects (`Box<dyn Repo>`, `&dyn Repo`, `Arc<dyn Repo>`). One vtable per concrete type, one heap allocation for the box.

The team's prevailing experience is that `Box<dyn …>` is reached for too quickly — every "I need an interface" instinct ends up costing inlining and surfaces type errors only at the call site rather than in the trait definition.

## Decision

**Static dispatch is the default.** `Box<dyn Trait>`, `Arc<dyn Trait>`, `&dyn Trait` are prohibited in production code unless one of the explicit escape hatches below applies.

### Patterns we use

- Use cases are `struct UseCase<R, F, C> where R: …, F: …, C: …`.
- Heterogeneous collections (event handlers, command handlers) are expressed as `enum` with `enum_dispatch`.
- Error trees are concrete `thiserror` enums; we do not `Box<dyn Error>`.
- Configuration is typed structs, not `HashMap<String, Box<dyn Any>>`.

### Allowed escape hatches

| Case                                                       | Reason it's allowed                                                       |
|------------------------------------------------------------|---------------------------------------------------------------------------|
| `tracing::Subscriber` and other foreign-trait requirements | We do not own those traits; the upstream uses `dyn`.                      |
| Test fixtures comparing N concrete impls of one trait      | Test-only, `#[cfg(test)]`-gated.                                          |
| `BoxFuture` from `futures` when no other way exists        | Only after a comment justifying why a generic version is infeasible.      |

### Enforcement

- `clippy::dyn_trait` is a warning in CI (we can't enable it as deny because of the foreign-trait exceptions). PR reviewers must check that any new `dyn` use is in the allow list.
- `cargo deny` bans depending on libraries whose API forces boxed traits when a static alternative exists.

## Rationale

- **Performance**: inlining + no per-call allocation. Notification rendering and translator lookups happen thousands of times per minute on busy servers.
- **Error locality**: a `where R: TaskRepository` bound on a use case surfaces a bound mismatch at the construction site, not deep inside a `Box<dyn>` call chain.
- **Easier reasoning**: the same compiler that validates the generic bound will tell us why a fake repository can't be substituted for the real one.
- **Better tooling**: `rust-analyzer` jumps into generic impls; `dyn` calls are harder to follow.

## Costs

- Slightly longer compile times due to monomorphisation. We accept this; release binaries still compile in well under 5 minutes.
- More verbose function signatures. Mitigated by type aliases at composition sites in `discord_bot`.
- Some heterogeneous patterns (a `Vec<Box<dyn Handler>>`) require slightly different shapes (a `Vec<AnyHandler>` enum). The `enum_dispatch` crate keeps this almost free.

## Considered alternatives

- **All `Box<dyn>`**: easier wiring but every code path pays the vtable + alloc tax. Rejected.
- **No abstraction at all**: would couple use cases to SeaORM and Twilight. Rejected; it kills our crate split (see ADR 0001).
- **A purpose-built DI framework**: rejected as YAGNI; the stdlib's type system is the DI framework.
