# Quickstart for Discord Server Administrators

> Audience: server owners and members with the "Manage Server" permission.
> Time required: about 5 minutes once the bot has been added.

## What this guide does

- Registers a user role with the bot.
- Sets the project (forum), active-tasks channel, notification channel, and watched category.
- Picks idle-detection thresholds.
- Picks a locale.

## 0. Prerequisites

- The bot is already in your server (ask your operator for the invite URL).
- You have the "Manage Server" permission.
- Have ready:
  - A **user role** to grant to people who will operate tasks.
  - A **project forum channel** (create one if needed).
  - An **active-tasks text channel** owned by the bot. `@everyone` must not be able to "Send Messages" here; the bot role needs "Send Messages", "Manage Messages", "Read Message History".
  - A **notification text channel** for stale-task pings.
  - Optionally, a **watched category** for channel-name counting and auto-task generation.

## 1. Configure

The fastest path is the interactive `/setup` wizard. The individual commands are below.

```
/config set role role: @users
/config set forum channel: #project-a
/config set active-channel channel: #active-tasks
/config set notify-channel channel: #task-notifications
/config set watch-category category: ongoing-tickets
/config set idle-threshold state: not_started days: 3
/config set idle-threshold state: in_progress days: 5
/config set notify-cooldown hours: 24
/config set locale value: en-US
```

All `/config` commands require the "Manage Server" permission.

## 2. Patterns (optional)

Inside the watched category, you can declare regex-based name patterns that either get counted or auto-generate tasks.

```
/config set pattern kind: counter name: ticket pattern: ^ticket-.*
/config set pattern kind: counter name: closed pattern: ^closed-.*
/config set pattern kind: counter name: temp pattern: ^temp-.*
/config set pattern kind: trigger name: ticket pattern: ^ticket-.*
```

- `counter` — runnable through `/count channels pattern: ticket`.
- `trigger` — when a channel matching the pattern is created inside the watched category, the bot opens a new forum thread (task) for it.

## 3. Verify configuration

```
/config show
```

Unset fields are highlighted in red. Once everything is green, you are ready to roll.

## 4. Smoke test

1. Create a new thread in the project forum.
2. Within a few seconds, the bot's message in the active-tasks channel should update.
3. Move the thread tag to the "In progress" equivalent.
4. The active-tasks message updates again.

## 5. Tell your users

User-facing guide: [docs/en/user/how-to-use.md](../user/how-to-use.md). Share the link with everyone who holds the user role.

## 6. Troubleshooting basics

- Want to change a setting? Use `/config set <key>`. Use `/config reset <key>` to clear.
- Need to swap the user role? Re-run `/config set role`.
- Need a full wipe? Ask your operator for a DB-level reset.
- Bot is silent? See the operator's guide: [docs/en/operator/deploy-docker.md](../operator/deploy-docker.md).

## 7. Permission summary

| Role                    | Commands available                                 |
|-------------------------|----------------------------------------------------|
| Server administrator    | `/config`, `/setup`, `/count`, `/task`             |
| User role               | `/task list`, `/task transition`, `/task assign`   |
| Anyone else             | Ignored (the bot replies with a polite error)      |
