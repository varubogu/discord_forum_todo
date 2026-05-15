# User Guide

> Audience: people who hold the "user role" — those who actually work tasks.

## 1. Day-to-day flow

1. Open a project-forum thread (= task), or create a new one.
2. When you start working on it, run `/task transition state: in_progress` inside the thread.
3. When it's done, run `/task transition state: done`.
4. The task disappears from the active list. Completed tasks remain searchable in the forum archive.

## 2. Commands

Run these inside a forum thread.

| Command                                            | What it does                                |
|----------------------------------------------------|---------------------------------------------|
| `/task transition state: not_started`              | Move back to "not started"                  |
| `/task transition state: in_progress`              | Mark as "in progress"                       |
| `/task transition state: done`                     | Mark as complete                            |
| `/task transition state: cancelled`                | Cancel (explain why in the thread body)     |
| `/task list`                                       | List tasks you should care about (ephemeral)|
| `/task assign user: @someone`                      | Assign someone (optional)                   |

## 3. Notifications

If a task stays in the same state for too long (your admin picks the threshold), the user role gets pinged in the notification channel. The whole role gets pinged, so try not to let things rot.

The re-ping interval is set by `notify-cooldown` (admin choice). Default: 24 hours.

## 4. The active-tasks channel

Your server has a dedicated "active tasks" channel.

- Only the bot writes there. Don't post.
- Lists every "not started" and "in progress" task.
- Updates within ~1 minute of any state change.

## 5. Common mistakes

- **A command errors out** — you may not have the user role. Ask your admin.
- **State changed but the list didn't refresh** — wait 30–60 seconds. Still nothing? Ping the admin.
- **Want to see completed tasks** — open the forum archive.

## 6. Auto-generated tasks

If your admin configured a watched category and a trigger pattern, certain channels created inside that category will spawn a forum task automatically. Don't panic if tasks "appear" on their own; just review the content before starting work.
