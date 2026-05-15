# 開発者向け — 拡張機能としての再利用

> 対象: 自前の Discord Bot に `discord_forum_todo` のタスク管理機能を組み込みたい開発者。

このプロジェクトのクレートは「素材」として設計されています。`discord_bot` バイナリ全体を取り込む必要はなく、必要な層だけ取り込めます。

## 1. 取り込み方の選択肢

### A. ドメインだけ欲しい

`core` だけ依存させてください。トランスポートと永続化は自前で実装します。

```toml
[dependencies]
discord_forum_todo_core = { git = "https://github.com/varubogu/discord_forum_todo", package = "core" }
```

`TaskRepository`、`GuildSettingsRepository`、`Translator`、`Clock` トレイトを自前で実装し、ユースケースに渡します。

### B. ドメイン + 永続化アダプタが欲しい

`core` + `core_infra` を依存。SeaORM をそのまま使える環境なら最も楽です。

```toml
[dependencies]
discord_forum_todo_core       = { git = "..." , package = "core" }
discord_forum_todo_core_infra = { git = "..." , package = "core_infra" }
```

### C. 全部欲しい（Discord 向け）

```toml
[dependencies]
discord_forum_todo_core               = { git = "..." , package = "core" }
discord_forum_todo_core_infra         = { git = "..." , package = "core_infra" }
discord_forum_todo_discord_task       = { git = "..." , package = "discord_task" }
discord_forum_todo_discord_task_infra = { git = "..." , package = "discord_task_infra" }
```

自前の `main.rs` で Twilight クライアントを起動し、`discord_task` のコマンド/イベントハンドラを自分のディスパッチャに登録します。

## 2. ハンドラの登録例

```rust
use discord_forum_todo_discord_task::{commands, events};

let create_task = commands::TaskTransition::new(task_repo.clone(), clock.clone(), translator.clone());
let stale_handler = events::ThreadUpdateHandler::new(task_repo.clone(), clock.clone());

router.register_command("/task", create_task);
router.register_event(events::AnyEventHandler::ThreadUpdate(stale_handler));
```

`router` は呼び出し側 Bot が用意する任意のディスパッチャ。`enum_dispatch` で生成された enum 経由で静的派遣されます。

## 3. ポート差し替え

たとえば「タスクストレージは Redis にしたい」場合、`TaskRepository` を自分で実装するだけです。`core_infra` には触らず、自分のクレートに `RedisTaskRepository` を作って use case の型パラメータに渡します。`core` も `discord_task` も再コンパイル不要です（生成型が違うだけ）。

## 4. マイグレーション

`core_infra` を再利用する場合、`migration` クレートのマイグレーションをあなたの DB に適用する必要があります。マイグレーションは追記式で、互換性のない変更は major バージョン bump に紐づきます。

```bash
cargo run -p discord_forum_todo_migration -- up
```

独自のマイグレーションと混ぜるときは、テーブル名衝突に注意してください（プレフィックス変更機能は今のところ提供していません — issue を立ててください）。

## 5. テストヘルパ

`core/tests/support/` に公開されている `FakeTaskRepository`, `FakeGuildSettingsRepository`, `FakeClock`, `FakeTranslator` を `dev-dependencies` で取り込んで自前のテストでも使えます。

```toml
[dev-dependencies]
discord_forum_todo_core = { git = "..." , package = "core", features = ["test-support"] }
```

## 6. ABI と SemVer ポリシー

- `0.x` の間: マイナーバージョン bump で破壊的変更が入り得ます。`Cargo.toml` ではマイナー固定 (`= 0.2.0`) を推奨。
- `1.0.0` 以降: SemVer 厳守。破壊的変更はメジャー bump。

## 7. 静的ディスパッチ前提

このクレート群は `Box<dyn>` を使いません。ジェネリクスで構築されているため、再エクスポートする型シグネチャは多少冗長になります。型エイリアスを自前で切ると楽です。

```rust
type MyCreateTask = discord_forum_todo_core::usecases::CreateTaskFromThread<
    RedisTaskRepository,
    MyForumService,
    SystemClock,
>;
```
