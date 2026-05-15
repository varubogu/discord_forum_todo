# 01 Ports and Adapters

## 1. Definitions

- **Port** = a Rust `trait` that the domain (or an upper adapter layer) depends on.
- **Adapter** = a concrete type that implements a port. Lives outside the layer that owns the port.

## 2. Where each port lives

| Port                          | Owner crate         | Why                                                                                                                      |
|-------------------------------|---------------------|--------------------------------------------------------------------------------------------------------------------------|
| `TaskRepository`              | `core`              | Domain-level data needs (CRUD, state transitions, stale query). Same port can back Discord and future WebUI.             |
| `GuildSettingsRepository`     | `core`              | Conceptually domain configuration. Both Discord and future WebUI need to read it.                                        |
| `ActiveMessageRepository`     | `core`              | Persisted artifact of the domain (which message represents the active task list).                                        |
| `ChannelPatternRepository`    | `core`              | Persisted matching rules.                                                                                                |
| `Translator`                  | `core`              | i18n is needed by every layer, not just Discord.                                                                         |
| `Clock`                       | `core`              | Pure time abstraction.                                                                                                   |
| `ForumService`                | `discord_task`      | Forum thread creation is a Discord-specific concern. The trait lives where Discord lives.                                |
| `ChannelInspector`            | `discord_task`      | Enumerates Discord channels in a category, matches patterns. Discord-specific.                                           |
| `ActiveMessageService`        | `discord_task`      | Sends/edits the active-tasks message via Twilight HTTP.                                                                  |
| `TaskNotifier`                | `discord_task`      | Posts mentions to the notify channel.                                                                                    |

Rule of thumb: **if the port has at least one non-Discord caller (current or planned), it belongs in `core`.** Otherwise it belongs in `discord_task`.

## 3. Where each adapter lives

| Adapter                         | Implements                                | Crate                  |
|---------------------------------|-------------------------------------------|------------------------|
| `SeaTaskRepository`             | `TaskRepository`                          | `core_infra`           |
| `SeaGuildSettingsRepository`    | `GuildSettingsRepository`                 | `core_infra`           |
| `SeaActiveMessageRepository`    | `ActiveMessageRepository`                 | `core_infra`           |
| `SeaChannelPatternRepository`   | `ChannelPatternRepository`                | `core_infra`           |
| `FluentTranslator`              | `Translator`                              | `core_infra`           |
| `SystemClock`                   | `Clock`                                   | `core_infra`           |
| `TwilightForumService`          | `ForumService`                            | `discord_task_infra`   |
| `TwilightChannelInspector`      | `ChannelInspector`                        | `discord_task_infra`   |
| `TwilightActiveMessageService`  | `ActiveMessageService`                    | `discord_task_infra`   |
| `TwilightTaskNotifier`          | `TaskNotifier`                            | `discord_task_infra`   |

## 4. Use case wiring

Use cases are generic over their ports. All ports are passed by value (the caller owns one instance per worker / per request as appropriate). `Clone` is required only for ports we want to share across tasks; SeaORM's `DatabaseConnection` is cheap to clone.

```rust
pub struct DetectStaleTasks<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    repo: R,
    clock: C,
}

impl<R, C> DetectStaleTasks<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    pub fn new(repo: R, clock: C) -> Self { Self { repo, clock } }

    pub async fn run(&self, guild_id: GuildId) -> Result<Vec<StaleTask>, DetectError> {
        // pure orchestration
    }
}
```

## 5. Heterogeneous collections

We use `enum_dispatch` to keep static dispatch even when we hold a list of "any handler".

```rust
#[enum_dispatch]
pub trait EventHandler {
    async fn handle(&self, event: &Event, ctx: &EventContext) -> Result<(), HandlerError>;
}

#[enum_dispatch(EventHandler)]
pub enum AnyEventHandler<R, S>
where
    R: TaskRepository,
    S: GuildSettingsRepository,
{
    ThreadCreate(ThreadCreateHandler<R, S>),
    ThreadUpdate(ThreadUpdateHandler<R>),
    ChannelCreate(ChannelCreateHandler<R, S>),
}
```

Adding a new handler = adding a variant to the enum. The compiler enforces exhaustiveness when we route events.

## 6. Async traits

We use the standard `async fn` in traits (stable since Rust 1.75). No `#[async_trait]` macro; we want generic, statically-dispatched async without an allocator per call.

```rust
pub trait TaskRepository: Send + Sync {
    async fn fetch_active(&self, guild_id: GuildId) -> Result<Vec<Task>, TaskRepoError>;
    async fn insert(&self, task: NewTask) -> Result<Task, TaskRepoError>;
    // ...
}
```

When auto-trait leakage (`Send`) becomes an issue we will use `return_position_impl_trait_in_trait` with explicit bounds rather than reverting to boxed futures.

## 7. Testing strategy

- **Domain unit tests** in `core`: write hand-rolled fakes (small structs with `Mutex<HashMap>` inside) implementing the ports. `mockall` is used selectively where assertion ergonomics outweigh the macro cost.
- **Adapter integration tests** in `core_infra` and `discord_task_infra`: spin up real PostgreSQL via `testcontainers`. Twilight HTTP calls are intercepted by `wiremock` when needed.
- **End-to-end** (smoke) tests live in `discord_bot/tests/`. They start the bot against a sandbox guild with a real Discord test token (CI-only, behind a feature flag).

## 8. Anti-patterns

- ❌ A port returning `Box<dyn Trait>`.
- ❌ A use case storing `Box<dyn Repository>`.
- ❌ A function taking `impl Trait` and immediately boxing it.
- ❌ An adapter that imports from another adapter (adapters know only the port and their own infra dependency).
- ❌ `core` importing `twilight-*` or `sea-orm`. Validated by `cargo deny check bans` rules.
