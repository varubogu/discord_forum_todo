# discord_forum_todo

> Discord forum channels as a task board, in production-grade Rust.
> 日本語版: [docs/ja/README.md](./docs/ja/README.md)

A Discord bot that maps `forum thread = task` and offers:

- A single bot-owned "active tasks" message that auto-updates on state changes.
- Stale-task detection with role mention in a configurable notification channel.
- Watched-category channel-name patterns for counting and auto-task generation.
- Per-guild slash-command configuration; no config files to edit.
- Japanese / English UI.
- Clean architecture with reusable crates so other bots can drop in our task model.

## Quick links

- **User guide** — [English](./docs/en/user/how-to-use.md) / [日本語](./docs/ja/user/how-to-use.md)
- **Admin quickstart** — [English](./docs/en/admin/quickstart.md) / [日本語](./docs/ja/admin/quickstart.md)
- **Operator (Docker) guide** — [English](./docs/en/operator/deploy-docker.md) / [日本語](./docs/ja/operator/deploy-docker.md)
- **Developer (extension) guide** — [English](./docs/en/developer/extend-bot.md) / [日本語](./docs/ja/developer/extend-bot.md)
- **Architecture & ADRs** — [docs/architecture/](./docs/architecture/)

## Workspace

| Crate                  | Role                                                                  |
|------------------------|------------------------------------------------------------------------|
| `core`                 | Pure domain: entities, ports (traits), use cases (generic).            |
| `core_infra`           | SeaORM repositories, Fluent translator, system clock.                  |
| `discord_task`         | Twilight-based slash commands and event handlers.                      |
| `discord_task_infra`   | Twilight-backed adapters and Discord-specific storage.                 |
| `discord_bot`          | Binary: gateway, scheduler, dependency injection.                      |
| `migration`            | `sea-orm-migration` project.                                           |
| `web_api`              | Reserved for the future Axum-based Web UI backend (not in workspace yet).|

## Tech stack

Rust (stable), Twilight, Tokio, PostgreSQL 16, SeaORM, fluent-rs, Axum (future), Docker, GitHub Actions.

## License

To be decided.
