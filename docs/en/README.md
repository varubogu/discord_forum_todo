# discord_forum_todo

A Discord bot that runs a forum channel as a lightweight task board.

- **One thread = one task**
- **Active-tasks summary** maintained as a single bot-owned message in a dedicated channel
- **Stale-task detection** pings the configured user role automatically
- **Channel-pattern monitoring** can spawn tasks from matching channels in a watched category
- **i18n** — Japanese and English

## Pick your guide

| Your role                       | Where to start                                       |
|---------------------------------|-------------------------------------------------------|
| User in a server                | [User guide](./user/how-to-use.md)                    |
| Server administrator            | [Admin quickstart](./admin/quickstart.md)             |
| Person hosting the bot          | [Docker deployment](./operator/deploy-docker.md)      |
| Developer extending the bot     | [Reusing the crates](./developer/extend-bot.md)       |

## Architecture documentation

In-depth design docs (architecture, data model, scheduler, i18n, ADRs) live under [`docs/architecture/`](../architecture/) in English.
