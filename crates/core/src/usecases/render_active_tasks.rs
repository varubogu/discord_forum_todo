//! Pure rendering of the active-tasks message. No I/O. The output is a list of
//! pages — Discord enforces per-message size limits, so callers may have to
//! send each page as its own message.

use crate::domain::{Task, TaskState};
use crate::i18n::{Locale, Translator};
use chrono::{DateTime, Utc};

/// Maximum characters per page. Discord allows 2000 in a plain message; we
/// reserve a small buffer for the trailing newline.
pub const PAGE_MAX_CHARS: usize = 1900;

/// A rendered, ready-to-send active-tasks payload split into Discord-sized pages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveMessageContent {
    pub pages: Vec<String>,
}

impl ActiveMessageContent {
    /// SHA-256 of the concatenated pages plus the locale tag, used to skip
    /// no-op edits.
    pub fn hash(&self, locale: &Locale) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        for page in &self.pages {
            h.update(page.as_bytes());
            h.update([0u8]);
        }
        h.update(locale.0.to_string().as_bytes());
        h.finalize().into()
    }
}

/// Render the message body for the given tasks. The function is generic over
/// the translator so unit tests can pass a fake without setting up Fluent.
#[allow(clippy::module_name_repetitions)]
pub fn render_active_tasks<T: Translator>(
    tasks: &[Task],
    locale: &Locale,
    translator: &T,
    now: DateTime<Utc>,
) -> ActiveMessageContent {
    let header_title = translator.t(locale, "active-tasks-header");
    let header_meta = format!("(updated <t:{}:R>, locale: {})", now.timestamp(), locale.0);

    let mut not_started: Vec<&Task> = tasks
        .iter()
        .filter(|t| t.state == TaskState::NotStarted && t.deleted_at.is_none())
        .collect();
    let mut in_progress: Vec<&Task> = tasks
        .iter()
        .filter(|t| t.state == TaskState::InProgress && t.deleted_at.is_none())
        .collect();
    not_started.sort_by_key(|t| t.last_state_changed_at);
    in_progress.sort_by_key(|t| t.last_state_changed_at);

    let mut sections = vec![format!("__**{header_title}**__ {header_meta}\n")];
    if !not_started.is_empty() {
        sections.push(format!(
            "**{}**\n",
            translator.t(locale, "active-tasks-section-not-started")
        ));
        for t in &not_started {
            sections.push(format_task_line(t));
        }
        sections.push(String::new());
    }
    if !in_progress.is_empty() {
        sections.push(format!(
            "**{}**\n",
            translator.t(locale, "active-tasks-section-in-progress")
        ));
        for t in &in_progress {
            sections.push(format_task_line(t));
        }
        sections.push(String::new());
    }
    if not_started.is_empty() && in_progress.is_empty() {
        sections.push(translator.t(locale, "active-tasks-empty"));
    }

    let body = sections.join("\n");
    ActiveMessageContent {
        pages: paginate(&body),
    }
}

fn format_task_line(t: &Task) -> String {
    format!(
        "- [{}](https://discord.com/channels/{}/{}) <t:{}:R>",
        t.title.replace(['[', ']'], ""),
        t.guild_id.0,
        t.thread_id.0,
        t.last_state_changed_at.timestamp()
    )
}

fn paginate(body: &str) -> Vec<String> {
    if body.len() <= PAGE_MAX_CHARS {
        return vec![body.to_string()];
    }
    let mut pages = Vec::new();
    let mut current = String::new();
    for line in body.lines() {
        if current.len() + line.len() + 1 > PAGE_MAX_CHARS {
            if !current.is_empty() {
                pages.push(std::mem::take(&mut current));
            }
            // Lines longer than a page get split arbitrarily.
            if line.len() > PAGE_MAX_CHARS {
                for chunk in line.as_bytes().chunks(PAGE_MAX_CHARS) {
                    pages.push(String::from_utf8_lossy(chunk).into_owned());
                }
                continue;
            }
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }
    if !current.is_empty() {
        pages.push(current);
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{GuildId, Task, TaskId, ThreadId};
    use chrono::TimeZone;

    struct StubTranslator;
    impl Translator for StubTranslator {
        fn t(&self, _locale: &Locale, key: &str) -> String {
            match key {
                "active-tasks-header" => "Active tasks".into(),
                "active-tasks-section-not-started" => "Not started".into(),
                "active-tasks-section-in-progress" => "In progress".into(),
                "active-tasks-empty" => "(no active tasks)".into(),
                other => other.into(),
            }
        }
    }

    fn task(id: i64, state: TaskState, title: &str) -> Task {
        let t = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
        let thread = 100u64 + u64::try_from(id).expect("test ids are non-negative");
        Task {
            id: TaskId(id),
            guild_id: GuildId(1),
            project_id: None,
            thread_id: ThreadId(thread),
            title: title.to_string(),
            state,
            created_at: t,
            last_state_changed_at: t,
            last_notified_at: None,
            deleted_at: None,
        }
    }

    #[test]
    fn renders_empty_message() {
        let now = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
        let c = render_active_tasks(&[], &Locale::fallback(), &StubTranslator, now);
        assert_eq!(c.pages.len(), 1);
        assert!(c.pages[0].contains("(no active tasks)"));
    }

    #[test]
    fn renders_two_sections() {
        let now = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
        let c = render_active_tasks(
            &[
                task(1, TaskState::NotStarted, "alpha"),
                task(2, TaskState::InProgress, "beta"),
                task(3, TaskState::Done, "gamma"), // filtered out
            ],
            &Locale::fallback(),
            &StubTranslator,
            now,
        );
        let body = c.pages.join("\n");
        assert!(body.contains("Not started"));
        assert!(body.contains("In progress"));
        assert!(body.contains("alpha"));
        assert!(body.contains("beta"));
        assert!(!body.contains("gamma"));
    }

    #[test]
    fn hash_is_stable_for_same_input() {
        let now = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
        let a = render_active_tasks(
            &[task(1, TaskState::NotStarted, "a")],
            &Locale::fallback(),
            &StubTranslator,
            now,
        );
        let b = render_active_tasks(
            &[task(1, TaskState::NotStarted, "a")],
            &Locale::fallback(),
            &StubTranslator,
            now,
        );
        assert_eq!(a.hash(&Locale::fallback()), b.hash(&Locale::fallback()));
    }
}
