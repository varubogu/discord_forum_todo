# 初期構築プランと進捗

> 作成: 2026-05-14 / 最終更新: 2026-05-16
> プロジェクト: `discord_forum_todo` — Discord フォーラムをタスクボードとして運用する Rust 製 Bot

このファイルは初期構築の全体計画と現時点の進捗を 1 箇所にまとめたものです。詳細な設計ドキュメントは [`docs/architecture/`](../docs/architecture/) を参照してください。

---

## 1. プロジェクト概要

- **スレッド 1 つ = タスク 1 つ** のフォーラム連動 Todo Bot
- **アクティブタスク表示**: Bot 専用チャンネル内の 1 メッセージ（複数ページ対応）に集約、状態変化で自動更新
- **放置検知**: 数日（サーバーごとに設定可能）状態が変わらないタスクを通知チャンネルで「利用者」ロールへ自動メンション
- **監視カテゴリ**: 特定パターン名のチャンネル作成を検知して自動タスク化、また `/count` でパターン別チャンネル数を返す
- **i18n**: 日本語 / 英語（将来追加可能）
- **設定**: サーバーごとに全てスラッシュコマンドで完結

---

## 2. 採用技術スタック

| 領域 | 技術 |
|------|------|
| 言語 | Rust stable / edition 2021 / MSRV 1.82 |
| Discord | Twilight 0.16 (gateway, http, model, cache-inmemory, util) |
| 非同期ランタイム | tokio 1.40 (full) |
| DB | PostgreSQL 16 |
| ORM / Migration | SeaORM 1.1 / sea-orm-migration |
| i18n | fluent-rs 0.16 |
| Web (将来) | Axum + 静的サイト分離 |
| ロギング | tracing + tracing-subscriber (JSON) |
| エラー | thiserror（lib層）/ anyhow（バイナリ層のみ） |
| 静的ディスパッチ補助 | enum_dispatch（異種ハンドラ集合） |
| テスト | tokio-test, mockall, testcontainers, cargo-nextest |
| デプロイ | Docker + docker-compose (cargo-chef マルチステージ) |
| CI | GitHub Actions (fmt / clippy / test / deny / release) |

---

## 3. クレート構成

依存方向（厳守）:

```
core               → (なし)
core_infra         → core
discord_task       → core
discord_task_infra → discord_task, core_infra
discord_bot        → 上記すべて + tokio + tracing
web_api(将来)      → core, core_infra
migration          → sea-orm-migration のみ
```

| クレート | 役割 |
|----------|------|
| `crates/core` | 純粋ドメイン: エンティティ、ポート(trait)、ユースケース(ジェネリクス) |
| `crates/core_infra` | core のポート実装: SeaORM レポジトリ、Fluent 翻訳器、SystemClock |
| `crates/discord_task` | Twilight ベースのコマンド/イベントハンドラ + Discord 固有ポート |
| `crates/discord_task_infra` | discord_task のポート実装 + Discord 固有テーブル |
| `crates/discord_bot` | 実バイナリ: DI、Twilight gateway、scheduler、graceful shutdown |
| `crates/web_api` | （将来）Axum で WebUI バックエンド。現状は workspace 除外 |
| `migration` | sea-orm-migration プロジェクト |

ユーザー指定の連鎖 `core ← discord_task ← discord_bot` を維持しつつ、再利用性のため infrastructure を 2 つに分割。

---

## 4. 実装フェーズと進捗

凡例: ✅ 完了 / 🔄 進行中 / ⏳ 未着手

| Phase | 内容 | 状態 |
|-------|------|------|
| 1 | ドキュメント先行作成（アーキテクチャ + ADR + 4 軸 × 2 言語マニュアル + mdBook） | ✅ |
| 2 | Cargo workspace + 全クレート骨格 + Docker + CI + locales プレースホルダ | ✅ |
| 3 | core クレート: ドメイン + ポート + ユースケース + テストサポート + 単体/統合テスト | ✅ |
| 4 | migration（SeaORM）+ core_infra の SeaORM リポジトリ実装 + testcontainers 統合テスト | ✅ |
| 5 | discord_task: スラッシュコマンド/イベントハンドラ + Discord 固有ポート定義 + mock Twilight テスト | ✅ |
| 6 | discord_task_infra: ポート実装 + Discord 固有テーブル | ✅ |
| 7 | discord_bot: main、DI 配線、scheduler、graceful shutdown | ✅ |
| 8 | i18n 実装: fluent-rs ベースの Translator 実体、ロケール解決 + テスト | ⏳ |
| 9 | Docker 本番ビルド検証 + docker-compose で end-to-end 起動確認 | ⏳ |
| 10 | release workflow（GHCR push, GitHub Release ノート）+ v0.1.0 タグ | ⏳ |

---

## 5. Phase 1 成果物（完了）

### 5.1 アーキテクチャ（英語）

- [00 Overview](../docs/architecture/00-overview.md)
- [01 Ports and Adapters](../docs/architecture/01-ports-and-adapters.md)
- [02 Data Model](../docs/architecture/02-data-model.md)
- [03 Scheduler](../docs/architecture/03-scheduler.md)
- [04 i18n](../docs/architecture/04-i18n.md)
- [05 Active Message](../docs/architecture/05-active-message.md)

### 5.2 ADR

- [0001 Clean architecture](../docs/architecture/adr/0001-clean-architecture.md)
- [0002 Static dispatch](../docs/architecture/adr/0002-static-dispatch.md)
- [0003 PostgreSQL only](../docs/architecture/adr/0003-postgresql-only.md)

### 5.3 ユーザー向けマニュアル（日英ペア）

- 利用者向け: [ja](../docs/ja/user/how-to-use.md) / [en](../docs/en/user/how-to-use.md)
- 管理者向けクイックスタート: [ja](../docs/ja/admin/quickstart.md) / [en](../docs/en/admin/quickstart.md)
- 運用者向け Docker デプロイ: [ja](../docs/ja/operator/deploy-docker.md) / [en](../docs/en/operator/deploy-docker.md)
- 開発者向け拡張ガイド: [ja](../docs/ja/developer/extend-bot.md) / [en](../docs/en/developer/extend-bot.md)

### 5.4 書籍ビルド

- mdBook 設定: [ja/book.toml](../docs/ja/book.toml), [en/book.toml](../docs/en/book.toml)
- SUMMARY.md / README.md（各言語）
- ルート [README.md](../README.md)

---

## 6. Phase 2 成果物（完了）

- ルート [Cargo.toml](../Cargo.toml) で workspace 定義 + `[workspace.dependencies]` 一元管理 + `[workspace.lints]` で pedantic-clippy をプロジェクト全体に適用
- [rust-toolchain.toml](../rust-toolchain.toml), [deny.toml](../deny.toml), [.gitignore](../.gitignore), [.env.example](../.env.example)
- 各クレートの `Cargo.toml` + 最小 `src/lib.rs` / `src/main.rs`
- Docker: [Dockerfile](../docker/Dockerfile)（cargo-chef multi-stage）, [docker-compose.yml](../docker/docker-compose.yml), [docker-compose.dev.yml](../docker/docker-compose.dev.yml)
- CI: [.github/workflows/ci.yml](../.github/workflows/ci.yml)（fmt / clippy / test on PostgreSQL service / deny）
- locales プレースホルダ: [ja-JP/commands.ftl](../locales/ja-JP/commands.ftl), [en-US/commands.ftl](../locales/en-US/commands.ftl)

### Phase 2 検証

| 確認 | 結果 |
|------|------|
| `cargo check --workspace --all-targets` (RUSTFLAGS=-D warnings) | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` (pedantic) | ✅ |
| `cargo fmt --all -- --check` | ✅ |
| `cargo test --workspace` | ✅ 骨格段階で 0 件 |

---

## 7. Phase 3 成果物（完了）

### 7.1 ドメインエンティティ

| ファイル | 内容 |
|----------|------|
| [ids.rs](../crates/core/src/domain/ids.rs) | `GuildId`, `ChannelId`, `RoleId`, `ThreadId`, `MessageId`, `CategoryId`, `UserId` の `u64` ラッパ |
| [task.rs](../crates/core/src/domain/task.rs) | `Task`, `NewTask`, `TaskId`, `TaskState`（NotStarted/InProgress/Done/Cancelled）+ 状態遷移ルール |
| [project.rs](../crates/core/src/domain/project.rs) | `Project`, `NewProject`, `ProjectId` |
| [guild_settings.rs](../crates/core/src/domain/guild_settings.rs) | `GuildSettings`, `IdleThresholds`（デフォルト 3 日）, ready 判定メソッド |
| [channel_pattern.rs](../crates/core/src/domain/channel_pattern.rs) | `ChannelPattern`, `ChannelPatternKind`（Trigger / Counter） |
| [active_message.rs](../crates/core/src/domain/active_message.rs) | `ActiveMessage`（複数 message_ids 対応、SHA-256 ハッシュ） |

### 7.2 ポート trait（すべて `async fn` in trait、`Send + Sync`）

| ファイル | trait |
|----------|-------|
| [task_repository.rs](../crates/core/src/ports/task_repository.rs) | `TaskRepository` + `StaleQuery`, `StaleTaskRow` |
| [guild_settings_repository.rs](../crates/core/src/ports/guild_settings_repository.rs) | `GuildSettingsRepository` + `GuildSettingsUpdate` enum |
| [active_message_repository.rs](../crates/core/src/ports/active_message_repository.rs) | `ActiveMessageRepository` |
| [channel_pattern_repository.rs](../crates/core/src/ports/channel_pattern_repository.rs) | `ChannelPatternRepository` |
| [project_repository.rs](../crates/core/src/ports/project_repository.rs) | `ProjectRepository` |
| [clock.rs](../crates/core/src/ports/clock.rs) | `Clock` + `FakeClock`（常時公開） |

### 7.3 ユースケース（全て `struct Foo<R: Repo, C: Clock>` ジェネリクス）

| ファイル | 役割 |
|----------|------|
| [register_guild_settings.rs](../crates/core/src/usecases/register_guild_settings.rs) | `GuildSettingsUpdate` 1 件適用（`/config set …` 1 コマンドに対応） |
| [create_task_from_thread.rs](../crates/core/src/usecases/create_task_from_thread.rs) | フォーラム新規スレッドからタスク作成（idempotent） |
| [transition_task_state.rs](../crates/core/src/usecases/transition_task_state.rs) | 状態遷移 + `last_state_changed_at` 更新 |
| [detect_stale_tasks.rs](../crates/core/src/usecases/detect_stale_tasks.rs) | 放置タスク取得（実 SQL の `FOR UPDATE SKIP LOCKED` は infra 側に委譲） |
| [render_active_tasks.rs](../crates/core/src/usecases/render_active_tasks.rs) | 純粋関数。`ActiveMessageContent`（複数ページ + SHA-256 ハッシュ）を返す |

### 7.4 テストサポート

| ファイル | 内容 |
|----------|------|
| [test_support/](../crates/core/src/test_support/) | 全リポジトリの in-memory Fake + FakeTranslator |

### 7.5 テスト

- ユニットテスト 11 件（ドメイン + render_active_tasks）
- 統合テスト 7 件（[tests/usecase_*.rs](../crates/core/tests/) 4 ファイル）
- **合計 18 件すべて緑**

### Phase 3 検証

| 確認 | 結果 |
|------|------|
| `cargo fmt --check` | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` (pedantic) | ✅ |
| `cargo test --workspace` | ✅ 18 件パス |

---

## 8. 重要な設計判断（要点）

| 判断 | 根拠 |
|------|------|
| `Box<dyn Trait>` 禁止、ジェネリクス + `enum_dispatch` | 静的ディスパッチで inline・型エラー早期検出（[ADR-0002](../docs/architecture/adr/0002-static-dispatch.md)） |
| infrastructure を 2 クレートに分離 (core_infra / discord_task_infra) | 他 Bot からの再利用性。core_infra のみ持ち出して discord_task_infra を自前実装可能 |
| ID は core 内 newtype、Twilight 型は infra 側で変換 | core を Twilight 非依存に保つ |
| 全ポートは `async fn` (stable native async-in-trait)、`#[async_trait]` 不採用 | アロケータ非依存、ジェネリクス維持 |
| 設定は単一 `GuildSettingsUpdate` enum + 個別コマンド | audit_log と 1:1 対応、冪等運用 |
| `render_active_tasks` は純粋関数 + ハッシュ比較 | rate limit 節約、テスト容易 |
| スケジューラ: `FOR UPDATE SKIP LOCKED` | 将来の複数インスタンス展開対応 |
| テスト: testcontainers PostgreSQL のみ。SQLite 非採用 | INTERVAL / JSONB / SKIP LOCKED など PG 固有を使うため（[ADR-0003](../docs/architecture/adr/0003-postgresql-only.md)） |
| `test_support` モジュールは常時 public | 統合テストと downstream 両方から使用、release は LTO で除去 |
| docs を先に書いて実装に反映 | 仕様ズレをコードまでに防ぐ。他 Bot 開発者も読める |

---

---

## 8.5 Phase 4 成果物（完了）

### マイグレーション

| ファイル | 内容 |
|----------|------|
| [m20260514_000001_init.rs](../migration/src/m20260514_000001_init.rs) | guild_settings / projects / tasks / channel_patterns / active_messages テーブル + インデックス |
| [lib.rs](../migration/src/lib.rs) | `Migrator` に上記マイグレーション登録 |

### SeaORM エンティティ

| ファイル | テーブル |
|----------|---------|
| [entities/tasks.rs](../crates/core_infra/src/entities/tasks.rs) | `tasks` |
| [entities/projects.rs](../crates/core_infra/src/entities/projects.rs) | `projects` |
| [entities/guild_settings.rs](../crates/core_infra/src/entities/guild_settings.rs) | `guild_settings` |
| [entities/channel_patterns.rs](../crates/core_infra/src/entities/channel_patterns.rs) | `channel_patterns` |
| [entities/active_messages.rs](../crates/core_infra/src/entities/active_messages.rs) | `active_messages`（複合 PK、JSONB） |

### SeaORM リポジトリ

| ファイル | 実装ポート |
|----------|-----------|
| [repositories/sea_task_repository.rs](../crates/core_infra/src/repositories/sea_task_repository.rs) | `TaskRepository`（FOR UPDATE SKIP LOCKED 生 SQL） |
| [repositories/sea_guild_settings_repository.rs](../crates/core_infra/src/repositories/sea_guild_settings_repository.rs) | `GuildSettingsRepository` |
| [repositories/sea_project_repository.rs](../crates/core_infra/src/repositories/sea_project_repository.rs) | `ProjectRepository` |
| [repositories/sea_active_message_repository.rs](../crates/core_infra/src/repositories/sea_active_message_repository.rs) | `ActiveMessageRepository` |
| [repositories/sea_channel_pattern_repository.rs](../crates/core_infra/src/repositories/sea_channel_pattern_repository.rs) | `ChannelPatternRepository` |

### 統合テスト（testcontainers PostgreSQL）

| ファイル | テスト件数 |
|----------|-----------|
| [tests/repo_task.rs](../crates/core_infra/tests/repo_task.rs) | 8 件（stale 検知・soft delete・冪等性含む） |
| [tests/repo_guild_settings.rs](../crates/core_infra/tests/repo_guild_settings.rs) | 3 件 |
| [tests/repo_project.rs](../crates/core_infra/tests/repo_project.rs) | 3 件 |
| [tests/repo_active_message.rs](../crates/core_infra/tests/repo_active_message.rs) | 3 件 |
| [tests/repo_channel_pattern.rs](../crates/core_infra/tests/repo_channel_pattern.rs) | 5 件 |

### Phase 4 検証

| 確認 | 結果 |
|------|------|
| `cargo fmt --check` | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` (pedantic) | ✅ |
| `cargo test --workspace` | ✅ 22 件パス（Phase 3 の 18 件 + 新規 22 件 = 合計 40 件） |
| migration up → down → up 冪等性 | ✅ |

---

## 9.5 Phase 5 成果物（完了）

### Discord 固有ポート定義

| ファイル | 内容 |
|----------|------|
| [ports/mod.rs](../crates/discord_task/src/ports/mod.rs) | `ForumService`, `ChannelInspector`, `ActiveMessageService`, `TaskNotifier`, `InteractionResponder` の 5 ポートトレイト + 各エラー型 |

### スラッシュコマンドハンドラ

| ハンドラ | コマンド | 使用ユースケース |
|----------|---------|----------------|
| `ConfigSetHandler<R,C,Resp>` | `/config set` | `RegisterGuildSettings` |
| `TaskStateHandler<R,C,Resp>` | `/task done|cancel|…` | `TransitionTaskState` |
| `CountHandler<PatternR,Insp,Resp>` | `/count` | `ChannelPatternRepository` + `ChannelInspector` |

### ゲートウェイイベントハンドラ

| ハンドラ | イベント | 処理 |
|----------|---------|------|
| `ThreadCreateHandler<R,C>` | `ThreadCreate` | `CreateTaskFromThread` ユースケース |
| `ThreadDeleteHandler<R,C>` | `ThreadDelete` | `TaskRepository::soft_delete` |
| `ChannelCreateHandler<GsR,PatternR,ForumSvc>` | `ChannelCreate` | Trigger パターン一致時に `ForumService::create_thread` |

`AnyEventHandler<TaskR,C,GsR,PatternR,ForumSvc>` 静的ディスパッチ enum を定義（Phase 7 でコンクリート型を解決）

### テスト（11 件すべて緑）

| ファイル | テスト件数 |
|----------|-----------|
| [tests/common.rs](../crates/discord_task/tests/common.rs) | 共通フェイク（FakeForumService 等 5 種） |
| [tests/cmd_config_set.rs](../crates/discord_task/tests/cmd_config_set.rs) | 2 件 |
| [tests/cmd_task_state.rs](../crates/discord_task/tests/cmd_task_state.rs) | 2 件 |
| [tests/evt_thread_create.rs](../crates/discord_task/tests/evt_thread_create.rs) | 4 件 |
| [tests/evt_thread_delete.rs](../crates/discord_task/tests/evt_thread_delete.rs) | 3 件 |

### Phase 5 検証

| 確認 | 結果 |
|------|------|
| `cargo fmt --check` | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo test -p discord_forum_todo_discord_task` | ✅ 11 件パス |
| `cargo test -p discord_forum_todo_core` | ✅ 既存テスト維持 |

### 設計上の注意

- `GatewayEventHandler` は native `async fn`（`async-trait` 不使用）。`AnyEventHandler` は各バリアントを `await` する `async fn handle` でマッチするため `enum_dispatch` マクロは使わず手動実装
- `FakeTaskRepository::find_by_thread` を修正して soft-delete 済みタスクを除外（real DB と一貫性確保）
- `CreateTaskFromThread::repo`, `TransitionTaskState::repo` を `pub` にして adapter テストからの状態検査を可能に

---

## 10.5 Phase 6 成果物（完了）

### discord_task_infra: Twilight HTTP クライアントによるポート実装

| ファイル | 実装ポート |
|----------|-----------|
| [src/interaction_responder.rs](../crates/discord_task_infra/src/interaction_responder.rs) | `InteractionResponder`（ephemeral / public 返信） |
| [src/forum_service.rs](../crates/discord_task_infra/src/forum_service.rs) | `ForumService`（`create_forum_thread` + ZWS 初期メッセージ） |
| [src/channel_inspector.rs](../crates/discord_task_infra/src/channel_inspector.rs) | `ChannelInspector`（`guild_channels` + regex filter） |
| [src/task_notifier.rs](../crates/discord_task_infra/src/task_notifier.rs) | `TaskNotifier`（`<@&role> タスク "title" が放置されています (<#thread>)`） |
| [src/active_message_service.rs](../crates/discord_task_infra/src/active_message_service.rs) | `ActiveMessageService`（ハッシュ比較→skip / PATCH / POST / DELETE） |

全構造体に `#[derive(Clone)]`（Arc 保持のため低コスト）。

### wiremock テスト（Docker なしで全件パス）

| ファイル | テスト件数 |
|----------|-----------|
| [tests/svc_interaction_responder.rs](../crates/discord_task_infra/tests/svc_interaction_responder.rs) | 2 件 |
| [tests/svc_forum_service.rs](../crates/discord_task_infra/tests/svc_forum_service.rs) | 1 件 |
| [tests/svc_channel_inspector.rs](../crates/discord_task_infra/tests/svc_channel_inspector.rs) | 2 件 |
| [tests/svc_task_notifier.rs](../crates/discord_task_infra/tests/svc_task_notifier.rs) | 1 件 |
| [tests/svc_active_message.rs](../crates/discord_task_infra/tests/svc_active_message.rs) | 4 件（testcontainers 依存 → Docker 環境でパス） |

**重要な実装上の知見**: Twilight 0.16 の `proxy()` はホスト名のみ受け取る（スキームを含む URL 不可）。wiremock の `server.uri()` から `"http://"` を除去して渡す必要がある（`tests/common/mod.rs` 参照）。

### Phase 6 検証

| 確認 | 結果 |
|------|------|
| `cargo fmt --check` | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| wiremock テスト 6 件（interaction / forum / inspector / notifier） | ✅ |
| testcontainers テスト 4 件（active_message） | Docker 環境依存（ローカル Docker 未起動） |

---

## 10.6 Phase 7 成果物（完了）

### discord_bot: DI 配線・ゲートウェイ・スケジューラ・グレースフルシャットダウン

| ファイル | 役割 |
|----------|------|
| [src/di.rs](../crates/discord_bot/src/di.rs) | `AppState`, `ConcreteAnyEventHandler` 型エイリアス, `build_app_state/event_handlers/command_handlers` |
| [src/gateway.rs](../crates/discord_bot/src/gateway.rs) | `run_gateway_loop`（Twilight StreamExt + InMemoryCache）+ コマンドディスパッチ |
| [src/scheduler.rs](../crates/discord_bot/src/scheduler.rs) | 放置通知ループ + アクティブメッセージ同期ループ + `StubTranslator` |
| [src/main.rs](../crates/discord_bot/src/main.rs) | DB 接続・migration・Shard 生成・scheduler spawn・`tokio::select!` + SIGINT |

#### DI 型エイリアス

```rust
pub type ConcreteAnyEventHandler = AnyEventHandler<
    SeaTaskRepository, SystemClock,
    SeaGuildSettingsRepository, SeaChannelPatternRepository, TwilightForumService,
>;
```

コマンドハンドラも同様に型エイリアスで静的ディスパッチを明示。

#### コマンドディスパッチ

- `"config"` → `ConfigSetInput`（`set` サブコマンド + `field`/`value` オプション解析）
- `"task"` → チャンネル ID から thread_id 経由でタスク検索 → `TaskStateInput`
- `"count"` → `CountInput`（`pattern` オプション省略可）

#### スケジューラ

- `StubTranslator`: Phase 8 まで i18n をキーそのまま返すスタブ
- guild_settings entity を直クエリ (`active_channel_id IS NOT NULL`) で全サーバー取得

### Phase 7 検証

| 確認 | 結果 |
|------|------|
| `cargo build --workspace` | ✅ |
| `cargo fmt --check` | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| `cargo test --workspace`（Docker 不要テストのみ） | ✅ |

---

## 9. これからの作業（Phase 8 以降）

- Phase 8: fluent-rs ベース Translator 実装、locale 解決順序テスト
- Phase 9: Docker production 起動検証
- Phase 10: GHCR push + release notes 自動化

---

## 10. 参考

- **計画書**（Claude Code の plan モードで書いた詳細プラン）: `C:\Users\toyos\.claude\plans\discord-todo-bot-cryptic-avalanche.md`
- **ルート README**: [../README.md](../README.md)
- **アーキテクチャ index**: [../docs/architecture/README.md](../docs/architecture/README.md)

---

## 11. 検証コマンド早見表

```powershell
# フォーマット
cargo fmt --all -- --check

# 静的解析（pedantic）
cargo clippy --workspace --all-targets -- -D warnings

# テスト全部
cargo test --workspace

# core クレートのみ
cargo test -p discord_forum_todo_core --all-targets

# マイグレーション up（Phase 4 以降）
cargo run -p discord_forum_todo_migration -- up

# ローカル DB のみ起動
docker compose -f docker/docker-compose.dev.yml up -d

# 本番フルスタック起動
docker compose -f docker/docker-compose.yml up -d --build

# cargo-deny（CI と同じ）
cargo deny check
```
