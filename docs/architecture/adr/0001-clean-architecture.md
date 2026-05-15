# ADR 0001 — Adopt clean (hexagonal) architecture with explicit crate split

- Status: Accepted
- Date: 2026-05-14
- Deciders: project owner

## Context

The product is a Discord bot today, but the owner has stated three forward-looking requirements:

1. A future Web UI may be added. The chosen Web framework is unsettled (Axum + static site separation has been picked as the leading direction).
2. The task-management logic must be reusable as an extension by other bots.
3. The Discord adapter should be independently usable by anyone who wants to drop our task model into their own bot binary.

A monolithic single-crate bot would satisfy "make it work" but force a rewrite the first time we add a Web UI or another bot tries to reuse the code.

## Decision

Split the project into the following Rust crates:

- `core` — pure domain. No I/O, no Discord, no DB. Defines entities, value objects, ports (traits), and use cases generic over ports.
- `core_infra` — adapters for ports that any consumer needs: SeaORM-backed repositories, Fluent translator, system clock.
- `discord_task` — Discord-specific consumer of `core`. Slash commands, event handlers, Discord-only ports (forum service, channel inspector, ...).
- `discord_task_infra` — adapters for `discord_task`'s ports, plus storage for Discord-specific tables.
- `discord_bot` — the binary. Wires everything up, runs the gateway and the scheduler.
- `web_api` — future Axum binary. Not in the workspace `members` yet. Will consume `core` + `core_infra`, never `discord_task*`.
- `migration` — `sea-orm-migration` project.

The dependency chain mirrors the owner's instruction `core ← discord_task ← discord_bot`. `core_infra` and `discord_task_infra` are siblings on the persistence axis so that an external Discord bot can re-use `discord_task` while bringing its own storage.

## Rationale

- **Reusability**: a third party can drop `core` + `core_infra` into a non-Discord application (e.g. our future Web UI) without inheriting any Twilight types.
- **Testability**: `core` is testable with hand-rolled fakes. We can iterate on domain rules without spinning up Postgres.
- **Boundary enforcement**: `cargo deny check bans` will be configured to forbid `twilight-*` and `sea-orm` from the `core` crate. Compile-time enforcement of the architecture.
- **Build times**: most of the build is in adapter crates, which change rarely. `cargo` incremental builds are friendlier with multiple smaller crates than a single mega-crate.
- **Backed by the spec**: the project owner explicitly required `core ← discord_task ← discord_bot` and a "third-party bot can use this as an extension" stance.

## Alternatives considered

### Single crate

Rejected. Mixing all layers blocks compile-time enforcement and means anyone who wants to reuse the domain takes Twilight as a dependency.

### Two crates (`core` + `discord_bot`)

Rejected. Better than monolith, but still bundles SeaORM into the binary crate, which a Web UI binary would need to duplicate.

### Trait objects everywhere (`Box<dyn Repo>`, etc.) instead of generics

Rejected. See [ADR 0002](./0002-static-dispatch.md).

### One infrastructure crate (`infra`) instead of two

Rejected. The plan review surfaced that another Discord bot wanting to reuse `discord_task` will probably want our SeaORM repos for tasks (in `core_infra`) but NOT our Discord-specific tables. Splitting `infra` into `core_infra` and `discord_task_infra` keeps that combination clean.

## Consequences

- More `Cargo.toml` files to maintain. Mitigated by a `[workspace.dependencies]` table that pins versions in one place.
- Tighter discipline around port placement. Documented in [01-ports-and-adapters.md](../01-ports-and-adapters.md).
- Slight onboarding overhead for new contributors. Mitigated by [00-overview.md](../00-overview.md) and `docs/{ja,en}/developer/`.
