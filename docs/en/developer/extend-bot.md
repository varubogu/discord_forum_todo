# Developer Guide — Reusing the Crates

> Audience: developers integrating `discord_forum_todo` into their own Discord bot or web app.

The crates are designed as building blocks. You do not have to take the whole `discord_bot` binary; pick the layer you need.

## 1. Integration options

### A. Domain only

Depend on `core`. Bring your own transport and storage.

```toml
[dependencies]
discord_forum_todo_core = { git = "https://github.com/varubogu/discord_forum_todo", package = "core" }
```

Implement `TaskRepository`, `GuildSettingsRepository`, `Translator`, `Clock` and hand them to the use cases.

### B. Domain + storage adapters

```toml
[dependencies]
discord_forum_todo_core       = { git = "...", package = "core" }
discord_forum_todo_core_infra = { git = "...", package = "core_infra" }
```

Easiest path when SeaORM and PostgreSQL fit your stack.

### C. The full Discord adapter

```toml
[dependencies]
discord_forum_todo_core               = { git = "...", package = "core" }
discord_forum_todo_core_infra         = { git = "...", package = "core_infra" }
discord_forum_todo_discord_task       = { git = "...", package = "discord_task" }
discord_forum_todo_discord_task_infra = { git = "...", package = "discord_task_infra" }
```

Run your own Twilight gateway in `main.rs` and register the command/event handlers from `discord_task` with your dispatcher.

## 2. Wiring example

```rust
use discord_forum_todo_discord_task::{commands, events};

let create_task = commands::TaskTransition::new(task_repo.clone(), clock.clone(), translator.clone());
let stale_handler = events::ThreadUpdateHandler::new(task_repo.clone(), clock.clone());

router.register_command("/task", create_task);
router.register_event(events::AnyEventHandler::ThreadUpdate(stale_handler));
```

`router` is whatever dispatcher your bot already has. Static dispatch goes through the `enum_dispatch`-generated enums.

## 3. Swapping a port

To put task storage on Redis, implement `TaskRepository` yourself in your own crate. You do not touch `core_infra`. The use cases receive your `RedisTaskRepository` through generic parameters; no recompilation of `core` or `discord_task` is needed (different monomorphisation, that's it).

## 4. Migrations

If you reuse `core_infra`, you also need to apply the migrations:

```bash
cargo run -p discord_forum_todo_migration -- up
```

Mixing our migrations with yours can collide on table names. A prefix option is not yet available — open an issue if you need one.

## 5. Test helpers

`core/tests/support/` exposes `FakeTaskRepository`, `FakeGuildSettingsRepository`, `FakeClock`, `FakeTranslator`. Enable the `test-support` feature to consume them from your own tests:

```toml
[dev-dependencies]
discord_forum_todo_core = { git = "...", package = "core", features = ["test-support"] }
```

## 6. SemVer policy

- `0.x`: minor bumps may break. Pin minor versions (`= 0.2.0`) in `Cargo.toml`.
- `1.0.0+`: strict SemVer. Breaking changes are major-only.

## 7. Static dispatch implications

These crates do not use `Box<dyn>`. Re-exported type signatures can get long; type aliases at your composition site help:

```rust
type MyCreateTask = discord_forum_todo_core::usecases::CreateTaskFromThread<
    RedisTaskRepository,
    MyForumService,
    SystemClock,
>;
```
