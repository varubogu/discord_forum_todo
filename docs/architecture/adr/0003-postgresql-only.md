# ADR 0003 — PostgreSQL only; no SQLite fallback

- Status: Accepted
- Date: 2026-05-14

## Context

SeaORM speaks SQLite, MySQL, and PostgreSQL. In a typical small bot project, supporting SQLite as the "easy local dev" backend is tempting:

- Single-binary dev environment.
- No container needed for tests.
- Faster CI loop.

But our design relies on several PostgreSQL features that SQLite either lacks or implements with materially different semantics:

| Feature                    | We use it for                                | SQLite story                                                  |
|----------------------------|----------------------------------------------|---------------------------------------------------------------|
| `INTERVAL` type            | per-guild idle thresholds, cooldowns         | not native; stored as text/ms, math is verbose                |
| `FOR UPDATE SKIP LOCKED`   | scheduler concurrency safety                 | not supported                                                 |
| `JSONB` + `jsonb_*` ops    | active-message page ids, audit_log payloads  | only `JSON` text, no operators                                |
| `gen_random_uuid()`        | request id correlation                       | needs an extension or manual UUID                             |
| Partial indexes            | `WHERE deleted_at IS NULL`                   | supported but semantics for nulls differ subtly               |
| `TIMESTAMP WITH TIME ZONE` | UTC contract for all timestamps              | SQLite has no real timestamp type                             |

If we tested against SQLite, the test suite would pass for code paths that fail on PostgreSQL in production. The whole point of testing is that the test reflects reality.

## Decision

- Runtime DB: **PostgreSQL 16+**.
- Test DB: **PostgreSQL 16 via `testcontainers-rs`**. Spin up one container per CI job, run migrations once, and use a per-test `BEGIN; … ROLLBACK;` for isolation.
- No SQLite anywhere — neither as test backend, nor as dev backend, nor behind a feature flag.

## Rationale

- **Test fidelity**: tests use the same DB engine as production. No quiet divergence.
- **Schema simplicity**: we are free to use PostgreSQL-native types (`INTERVAL`, `JSONB`) without inventing a portability layer.
- **Operational simplicity**: one engine, one set of operational documents, one set of failure modes.
- **`testcontainers` is fast enough**: container reuse + a single shared DB across all tests in a job keeps per-test overhead in the low milliseconds.

## Consequences

- New contributors need Docker (or a local PostgreSQL). Documented in `docs/{ja,en}/operator/deploy-docker.md` and `docs/{ja,en}/developer/`.
- We cannot run tests in environments without container support. CI is GitHub Actions where this is solved out of the box.
- `Cargo.toml`s use `sea-orm` with only the `sqlx-postgres` feature, never `sqlx-sqlite`. `cargo deny` confirms.

## Alternatives considered

- **Dual-target with feature flags**: would require two query strings for any non-portable feature. Rejected; the maintenance tax exceeds the benefit.
- **In-memory mock at the repository layer**: we already do this in `core` unit tests via hand-rolled fakes. The integration tests stay on real PostgreSQL.
