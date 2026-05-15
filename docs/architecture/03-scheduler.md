# 03 Scheduler — Stale-Task Detection

## 1. Goals

- Run every minute (configurable via `BOT_SCHEDULER_INTERVAL_SECS`, default 60).
- For each guild, find tasks whose `state IN ('not_started','in_progress')` and whose `last_state_changed_at` is older than the guild-specific threshold.
- Avoid double-notifying within `notify_cooldown`.
- Survive multiple bot instances (future deployment), restart cleanly, and never block a Discord event handler.

## 2. Implementation outline

The scheduler runs in `discord_bot/src/scheduler.rs` as a dedicated tokio task spawned at startup.

```text
loop {
    select! {
        _ = interval.tick() => run_one_tick().await,
        _ = shutdown_rx.recv() => break,
    }
}
```

### 2.1 Query

```sql
SELECT t.*,
       g.user_role_id,
       g.notify_channel_id,
       g.notify_cooldown,
       g.locale
FROM tasks t
JOIN guild_settings g ON g.guild_id = t.guild_id
WHERE t.deleted_at IS NULL
  AND t.state IN ('not_started', 'in_progress')
  AND g.notify_channel_id IS NOT NULL
  AND g.user_role_id IS NOT NULL
  AND t.last_state_changed_at < now() - CASE t.state
      WHEN 'not_started' THEN g.idle_threshold_not_started
      WHEN 'in_progress' THEN g.idle_threshold_in_progress
  END
  AND (t.last_notified_at IS NULL OR t.last_notified_at < now() - g.notify_cooldown)
ORDER BY t.guild_id, t.last_state_changed_at
FOR UPDATE OF t SKIP LOCKED
LIMIT :batch_size;
```

- `FOR UPDATE … SKIP LOCKED` lets multiple bot instances coexist without one stealing work the other is already sending.
- The lock is held for the duration of the **send**, so we keep the transaction short by sending one guild's batch per transaction.

### 2.2 Send fan-out

1. Group rows by `guild_id`.
2. Render one notification per guild listing all stale tasks (bullet list with thread links).
3. Send through `TaskNotifier::notify_stale(guild_id, message, role_mention)`.
4. On success: `UPDATE tasks SET last_notified_at = now() WHERE id = ANY($1)`; commit.
5. On Discord failure: retry within the same tick once (immediate retry handles transient 5xx). If still failing, ROLLBACK, increment a counter, and let the next tick try again. After 3 consecutive failures for the same row, write a `dead_letter` entry and mark `last_notified_at = now()` so we stop hammering.

### 2.3 Rate-limit awareness

`twilight-http` exposes rate-limit headers. The notifier uses `twilight-http`'s built-in rate limiter and respects 429 retry-after. Inside a tick we cap concurrent sends per guild at 1 (per-guild `tokio::sync::Mutex`) and total concurrent sends at 16 (a `tokio::sync::Semaphore`).

## 3. Time and testability

All `now()` reads in Rust go through `Clock::now()`. In tests, a `FakeClock` lets us advance virtual time:

```rust
let clock = FakeClock::new(start);
let scheduler = Scheduler::new(repo.clone(), notifier.clone(), clock.clone());
clock.advance(Duration::from_days(4));
scheduler.run_once().await.unwrap();
```

The SQL `now()` is replaced in tests by a wrapper that injects a known timestamp parameter. We keep one production path (SQL `now()`) and one test path (parameter) by feature-gated query string.

## 4. Idempotency

Two consecutive ticks must produce at most one notification per task. The combination of `last_notified_at` UPDATE + `FOR UPDATE SKIP LOCKED` + the cooldown predicate guarantees this even under crash-restart (we re-evaluate the predicate on every tick).

## 5. Failure modes

| Failure                              | Behaviour                                                                 |
|--------------------------------------|---------------------------------------------------------------------------|
| DB connection lost                   | Tick logs error, skips. Next tick retries. Pool reconnects automatically. |
| Discord API 5xx                      | Retry once, then dead-letter after 3 consecutive failures.                |
| Discord API 403 (lost permission)    | One log line, skip the guild for this tick, do not retry.                 |
| Notification channel deleted         | 404 — dead-letter the row; admins must rerun `/config set notify-channel`.|
| Thread deleted                       | Filtered out by the next forum sync; tasks marked `deleted_at`.           |

## 6. Metrics events

- `scheduler.tick.started`
- `scheduler.tick.completed` (with duration, guilds_processed, tasks_notified)
- `scheduler.tick.failed`
- `scheduler.notification.sent` (per task)
- `scheduler.notification.dead_letter`

These are emitted via `tracing::event!(target: "metrics", ...)` so a future Prometheus/OTLP layer can subscribe without code churn.

## 7. Bounded resource usage

- Each tick processes at most `BATCH_SIZE` (default 200) rows.
- If more than `BATCH_SIZE` are eligible, the next tick picks them up.
- Memory per tick is therefore bounded by `BATCH_SIZE × sizeof(StaleTaskRow)` (a few hundred KB at worst).

## 8. Open questions

- Should we expose `/admin trigger-scheduler` for manual runs? Deferred — operationally nice but adds an attack surface. Revisit if support pings need it.
- Per-task per-day cap on notifications? Currently controlled by `notify_cooldown`. Revisit if abuse reports surface.
