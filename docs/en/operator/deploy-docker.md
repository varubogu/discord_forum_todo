# Operator Guide — Docker Deployment

> Audience: operators (DevOps/SRE) hosting the bot.

## 1. Prerequisites

- A host with Docker 24+ and Docker Compose v2+ (VPS, your own machine, a k8s node — anything).
- A Discord bot **token** generated in the developer portal.
- A few gigabytes of disk for the database volume.

## 2. Minimum deploy

```bash
git clone https://github.com/varubogu/discord_forum_todo.git
cd discord_forum_todo
cp .env.example .env
$EDITOR .env   # set DISCORD_TOKEN, DATABASE_URL, RUST_LOG
docker compose -f docker/docker-compose.yml up -d
```

This brings up Postgres + the bot. Migrations apply automatically on first boot (the `migration` binary runs during container startup).

## 3. Environment variables

| Variable                          | Required | Default     | Description                                       |
|-----------------------------------|----------|-------------|---------------------------------------------------|
| `DISCORD_TOKEN`                   | ✓        | -           | Discord bot token                                 |
| `DATABASE_URL`                    | ✓        | -           | e.g. `postgres://bot:pass@postgres:5432/bot`      |
| `RUST_LOG`                        |          | `info`      | `tracing-subscriber` filter                       |
| `RUST_LOG_FORMAT`                 |          | `json`      | `pretty` for human-readable                       |
| `BOT_SCHEDULER_INTERVAL_SECS`     |          | `60`        | Stale-detection tick interval                     |
| `BOT_SCHEDULER_BATCH_SIZE`        |          | `200`       | Max rows per tick                                 |
| `BOT_SHUTDOWN_TIMEOUT_SECS`       |          | `30`        | Graceful shutdown deadline                        |
| `BOT_DEFAULT_LOCALE`              |          | `en-US`     | Fallback when guild has no locale set             |

## 4. Discord developer portal

1. https://discord.com/developers/applications → "New Application".
2. **Bot** tab → create a bot user, copy the token into `.env`.
3. **Privileged Gateway Intents**: neither `MESSAGE CONTENT` nor `SERVER MEMBERS` is required. `GUILD MEMBERS` is recommended.
4. **OAuth2 → URL Generator**: select `bot` + `applications.commands`. Permissions:
   - Send Messages
   - Manage Messages (needed to edit/delete the active-tasks message)
   - Read Message History
   - Use Slash Commands
   - Mention Everyone (for role mentions in stale notifications)
   - Create Public Threads
5. Invite the bot with the generated URL.

## 5. Backups

Back up the Postgres volume regularly with `pg_dump`:

```bash
docker exec -t discord_forum_todo-postgres-1 pg_dump -U bot -d bot > backup-$(date +%F).sql
```

Restore:

```bash
docker exec -i discord_forum_todo-postgres-1 psql -U bot -d bot < backup-2026-05-14.sql
```

## 6. Upgrades

```bash
git pull
docker compose -f docker/docker-compose.yml pull
docker compose -f docker/docker-compose.yml up -d
```

The `migration` service applies new migrations on startup. `up` is idempotent.

### PostgreSQL 16 to 18

The official `postgres:18` image moved its data directory.

- Default `PGDATA` is `/var/lib/postgresql/18/docker`.
- Persist the volume at `/var/lib/postgresql`. A mount at `/var/lib/postgresql/data` is rejected even when the volume is empty.

The Compose files use that mount. An existing PostgreSQL 16 volume (`discord_forum_todo_postgres-data`) cannot be reused by 18. Major versions do not start in place; dump with `pg_dump` and restore into a new 18 data directory.

1. Dump while the 16 container is still running.

```bash
docker exec -t discord_forum_todo-postgres-1 pg_dump -U bot -d discord_forum_todo > backup.sql
```

2. Stop the stack and remove the old volume after you have checked the dump. Attaching 16 files at the 18 mount point fails on `PG_VERSION`.

```bash
docker compose -f docker/docker-compose.yml down
docker volume rm discord_forum_todo_postgres-data
```

3. Start 18 and restore.

```bash
docker compose -f docker/docker-compose.yml up -d
docker exec -i discord_forum_todo-postgres-1 psql -U bot -d discord_forum_todo < backup.sql
```

## 7. Logs and metrics

- JSON logs go to stdout: `docker compose logs -f bot`.
- Structured output is easy to forward to Loki / Datadog / CloudWatch.
- Metrics are emitted as `tracing` events with `target: "metrics"`. A Prometheus exporter is planned.

## 8. Troubleshooting

| Symptom                                       | Action                                                                  |
|-----------------------------------------------|-------------------------------------------------------------------------|
| Bot won't start                               | `docker compose logs bot` — check token, DB URL.                        |
| `/config` isn't in the slash-command list     | Wait a few minutes after restart; Discord caches command registrations. |
| Active-tasks message never updates            | Confirm the bot has "Manage Messages" in that channel.                  |
| Migrations fail                               | Run `docker compose run migration up` manually for verbose output.      |
| Flood of duplicate notifications              | Increase `notify-cooldown`.                                             |

## 9. Scaling

- Up to ~hundreds of guilds: one instance is fine.
- Thousands of guilds: run multiple instances. The scheduler uses `FOR UPDATE SKIP LOCKED` so they won't double-send. Use Twilight's `ShardScheme` for gateway sharding.
