# 05 Active-Tasks Message Lifecycle

## 1. Goal

The active-tasks channel hosts **a single bot-owned conversation** that mirrors the current set of non-completed tasks. Users and other bots are not allowed to write in this channel (enforced via Discord permission setup, documented in `docs/{ja,en}/admin/quickstart.md`).

## 2. State

Persisted in `active_message`:

| guild_id | channel_id | message_ids                  | last_rendered_hash | updated_at |
|----------|------------|------------------------------|--------------------|------------|
| 123…     | 456…       | [987…]                       | <sha256>           | …          |
| 124…     | 459…       | [990…, 991…, 992…]           | <sha256>           | …          |

`message_ids` is an ordered array. Element 0 is the "header" message; subsequent elements are continuation pages when content exceeds Discord limits (single message ≤ 2 000 chars, or 6 000 in embeds).

## 3. Trigger points

The renderer runs on these events:

- `ThreadCreate` in the project forum
- `ThreadUpdate` (state change, archive, rename) for an existing task
- `ThreadDelete` for an existing task
- `/task transition` slash command
- `ChannelCreate` auto-task generation
- A successful scheduler tick that mutated `last_notified_at` (so the message can reflect "stale" badges)

We deliberately do **not** prefetch on startup. Prefetch on hundreds of guilds would burn Discord API quota for zero benefit; lazy refresh covers the realistic case.

## 4. Render flow

```rust
async fn refresh(guild_id: GuildId) -> Result<(), RenderError> {
    let _lock = per_guild_lock(guild_id).await;       // serialize per guild

    let tasks = task_repo.fetch_active(guild_id).await?;
    let locale = guild_settings.locale_for(guild_id).await?;
    let payload = render_active_tasks(&tasks, &locale, &translator); // pure fn

    let hash = sha256(&payload);
    let existing = active_message_repo.find(guild_id).await?;
    if existing.as_ref().map(|m| m.last_rendered_hash == hash).unwrap_or(false) {
        return Ok(()); // no-op, save the API call
    }

    let channel_id = guild_settings.active_channel_for(guild_id).await?;
    let new_message_ids = sync_messages(channel_id, existing.as_ref(), &payload).await?;

    active_message_repo.upsert(ActiveMessage {
        guild_id, channel_id, message_ids: new_message_ids, last_rendered_hash: hash,
        ..
    }).await?;

    Ok(())
}
```

### 4.1 `per_guild_lock`

A `tokio::sync::Mutex` per `GuildId` held inside a `dashmap::DashMap<GuildId, Arc<Mutex<()>>>`. We never hold this lock across an entire HTTP call sequence that exceeds 10 seconds; if rendering takes longer (rare, only with thousands of tasks) we will batch.

### 4.2 `sync_messages`

Pseudo:

```rust
let pages = split_payload(payload);                      // Vec<String>
match existing {
    None => {
        let mut ids = vec![];
        for page in pages { ids.push(http.create_message(channel_id).content(page).await?.id); }
        Ok(ids)
    }
    Some(existing) => {
        // 1. Verify the existing first message still lives
        let head_alive = match http.message(channel_id, existing.message_ids[0]).await {
            Ok(_) => true,
            Err(e) if is_404_or_403(&e) => false,
            Err(e) => return Err(e.into()),
        };
        if !head_alive {
            // re-create from scratch
            return sync_messages(channel_id, None, payload).await;
        }
        // 2. Edit existing pages we still have, create new ones, delete leftovers
        let mut ids = vec![];
        for (i, page) in pages.iter().enumerate() {
            match existing.message_ids.get(i) {
                Some(mid) => { http.update_message(channel_id, *mid).content(page).await?; ids.push(*mid); }
                None      => { ids.push(http.create_message(channel_id).content(page).await?.id); }
            }
        }
        for stale_mid in existing.message_ids.iter().skip(pages.len()) {
            let _ = http.delete_message(channel_id, *stale_mid).await; // best-effort
        }
        Ok(ids)
    }
}
```

## 5. Permissions

The admin docs require:

- The bot has `Send Messages`, `Manage Messages` (for delete), `Read Message History` in the active channel.
- Everyone else has `View Channel` but not `Send Messages`.
- The channel topic carries a comment: "Managed by discord_forum_todo. Do not post here."

## 6. Hashing

`last_rendered_hash` is SHA-256 over the concatenated UTF-8 bytes of all pages plus the locale. We include the locale to force a re-render when the guild changes its default locale.

## 7. Layout

Each page is roughly:

```
__**Active tasks**__ (last updated: 2026-05-14T10:00Z, locale: ja-JP)

**Not started**
- [Thread title](thread_link) — 3 days ago
- ...

**In progress**
- [Thread title](thread_link) — 1 day ago
- ...
```

Time deltas use the Discord native timestamp markup `<t:UNIX:R>` so each viewer sees them in their own timezone.

## 8. Failure modes

| Failure                                | Handling                                                                  |
|----------------------------------------|---------------------------------------------------------------------------|
| Active channel deleted                 | Render aborts with `RenderError::ChannelGone`; logged once; admins re-set.|
| Bot loses send permission              | 403 from HTTP; we log and skip until admins re-grant.                     |
| Hash check thrashes (concurrent edits) | per-guild lock prevents this.                                              |
| Active message manually deleted by an admin | 404 on next render → recreated transparently.                        |

## 9. Test plan

- Unit test `render_active_tasks` (pure fn) with golden snapshots per locale.
- Integration test against a stubbed `TwilightActiveMessageService` that records HTTP calls.
- Smoke test in `discord_bot/tests/` using a real test guild (gated behind `DISCORD_E2E_TOKEN`).
