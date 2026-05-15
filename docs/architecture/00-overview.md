# 00 Architecture Overview

> Audience: developers and architects working on `discord_forum_todo` or building extensions.
> Language: English. Localised user-facing guides live under `docs/ja/` and `docs/en/`.

## 1. Purpose

`discord_forum_todo` is a Discord bot that turns a forum channel into a lightweight task board.

- Each forum thread maps 1:1 to a task.
- A dedicated "active tasks" channel is rendered into a single bot-owned message that auto-updates as task state changes.
- Stale tasks (no state change for N days) are flagged and the configured user role is mentioned in a notification channel.
- A monitored category can be inspected for channel names matching configured patterns (`ticket*`, `closed*`, `temp*`, ...), and channels matching trigger patterns are converted into tasks automatically.

## 2. Architectural Style

Hexagonal / clean architecture. Domain logic lives in a pure `core` crate that knows nothing about Discord, the database, or the operating system. All side effects are reached through traits (ports). Adapters implement those traits.

```
+----------------+         +-------------------------+
|  discord_task  | <-uses- |     core (domain)       |
|  (commands &   |         |  - entities             |
|   events)      |         |  - value objects        |
+----------------+         |  - use cases (generic)  |
        ^                  |  - ports (traits)       |
        |                  +-------------------------+
        |                              ^
+----------------+                     |
| discord_bot    | --------------------+
|  (binary; DI,  |
|  scheduler,    |
|  gateway)      |
+----------------+
        ^
        | implements ports through
        |
+--------------------+   +--------------------+
| core_infra         |   | discord_task_infra |
| (SeaORM + Fluent)  |   | (Twilight-backed   |
|                    |   |  GuildSettings,    |
|                    |   |  ActiveMessage)    |
+--------------------+   +--------------------+

+----------+
| web_api  | (future; reuses core + core_infra; no Discord deps)
+----------+
```

## 3. Crate Layout

| Crate                  | Role                                                                                  | Depends on                                          |
|------------------------|---------------------------------------------------------------------------------------|-----------------------------------------------------|
| `core`                 | Pure domain: entities, value objects, ports (traits), use cases generic over ports.   | nothing outside `std`, `chrono`, `thiserror`, etc.  |
| `core_infra`           | SeaORM repositories, Fluent translator, `Clock` impl.                                  | `core`                                              |
| `discord_task`         | Twilight slash command handlers, event handlers, Discord-specific ports.              | `core` (+ twilight-*)                               |
| `discord_task_infra`   | SeaORM repos for Discord-specific tables (guild settings, active message, patterns).   | `discord_task`, `core_infra`                        |
| `discord_bot`          | Binary. Wires up dependencies, runs gateway, scheduler, graceful shutdown.            | all of the above + tokio + tracing                  |
| `web_api`              | Future Axum binary. Not in workspace `members` yet; reuses `core` + `core_infra`.     | `core`, `core_infra`                                |
| `migration`            | `sea-orm-migration` project that owns the schema.                                     | `sea-orm-migration`                                 |

The chain the user explicitly requested — `core ← discord_task ← discord_bot` — is preserved. `core_infra` and `discord_task_infra` are siblings on the persistence axis. They exist so that another Discord bot can re-use `discord_task` without inheriting our `discord_bot` binary or DB schema.

## 4. Static Dispatch Discipline

`Box<dyn Trait>`, `Arc<dyn Trait>`, `&dyn Trait` are **not** allowed in production code paths. Reasons:

- Use cases are constructed once at startup and called millions of times; vtable indirection has no upside here.
- Generics force the compiler to inline and verify, surfacing trait misuses earlier.
- Heterogeneous collections (event handlers, command handlers) are expressed as `enum`s with `enum_dispatch`.

Patterns we use:

```rust
pub struct CreateTaskFromThread<R, F, C>
where
    R: TaskRepository,
    F: ForumService,
    C: Clock,
{
    pub tasks: R,
    pub forum: F,
    pub clock: C,
}
```

```rust
#[enum_dispatch(EventHandler)]
pub enum AnyEventHandler {
    ThreadCreate(ThreadCreateHandler<...>),
    ThreadUpdate(ThreadUpdateHandler<...>),
    ChannelCreate(ChannelCreateHandler<...>),
}
```

See [ADR-0002](./adr/0002-static-dispatch.md) for the full rationale and the limited list of escape hatches.

## 5. Async Runtime

`tokio` with the `full` features. No alternative runtime. Reasons:

- Twilight, SeaORM, Axum, Reqwest, Tracing all target Tokio.
- A single runtime simplifies cancellation, timers, and tracing.

## 6. Error Strategy

- Each library crate (`core`, `core_infra`, `discord_task`, `discord_task_infra`, `migration`) defines its own error enum with `thiserror`. No `anyhow` inside libraries.
- The `discord_bot` binary uses `anyhow` only at the `main` boundary.
- All public functions return `Result<T, ThisCrateError>`; never `Result<T, Box<dyn Error>>`.

## 7. Observability

- Logging: `tracing` + `tracing-subscriber` with JSON output by default. Plain output when `RUST_LOG_FORMAT=pretty`.
- Spans always carry `guild_id`, `user_id`, `command`, `request_id` where applicable.
- Counters are surfaced as `tracing::event!(target: "metrics", ...)` so they can later be piped into Prometheus/OTLP without touching call sites.

## 8. Configuration

- Environment variables only (no config file). Loaded into a typed `BotConfig` struct via `figment`.
- Required: `DISCORD_TOKEN`, `DATABASE_URL`.
- Optional with defaults: `RUST_LOG`, `RUST_LOG_FORMAT`, `BOT_SCHEDULER_INTERVAL_SECS` (default 60), `BOT_SHUTDOWN_TIMEOUT_SECS` (default 30).

Per-guild settings (role, channels, thresholds, patterns) are stored in PostgreSQL and managed through slash commands.

## 9. Glossary

- **Project**: a forum channel that hosts task threads.
- **Task**: a single forum thread, mapped to a `Task` entity in the DB.
- **Active message**: the bot-owned message in the "active tasks" channel that mirrors current non-completed tasks.
- **User role** (a.k.a. "Daijin" in informal docs): a Discord role registered per guild. Members of this role can operate the bot. Stale-task notifications mention this role.
- **Idle threshold**: time after which a task in `NotStarted` or `InProgress` is considered stale. Two separate values per guild.
- **Notify cooldown**: minimum interval between two stale notifications for the same task.
- **Channel pattern**: a per-guild rule that matches a channel name. Two kinds: `Trigger` (creates a task on matching new channel) and `Counter` (counted by `/count`).

## 10. Where to go next

| You want to ...                            | Read                                                        |
|--------------------------------------------|-------------------------------------------------------------|
| understand trait placement                 | [01-ports-and-adapters.md](./01-ports-and-adapters.md)      |
| understand the DB schema                   | [02-data-model.md](./02-data-model.md)                      |
| understand stale-task detection            | [03-scheduler.md](./03-scheduler.md)                        |
| understand i18n resolution                 | [04-i18n.md](./04-i18n.md)                                  |
| understand the active message lifecycle    | [05-active-message.md](./05-active-message.md)              |
| revisit a design decision                  | [adr/](./adr/)                                              |
