# 運用者向け Docker デプロイガイド

> 対象: Bot をホスティングする運用者（DevOps / SRE 担当者）。

## 1. 用意するもの

- Docker 24 以上、Docker Compose v2 以上が動くサーバー（VPS、自前 PC、k8s ノードなど）。
- Discord 開発者ポータルで作成したアプリケーション（Bot ユーザー）の **トークン**。
- 永続化用ボリュームを置けるディスク（数 GB 程度で十分）。

## 2. 最短デプロイ

```bash
git clone https://github.com/varubogu/discord_forum_todo.git
cd discord_forum_todo
cp .env.example .env
$EDITOR .env   # DISCORD_TOKEN, DATABASE_URL, RUST_LOG などを入力
docker compose -f docker/docker-compose.yml up -d
```

これで Postgres + Bot コンテナが起動します。初回起動時にマイグレーションが自動適用される構成です（コンテナ起動時に `migration` バイナリが流れる）。

## 3. 環境変数

| 変数                            | 必須 | デフォルト       | 説明                                              |
|---------------------------------|------|------------------|---------------------------------------------------|
| `DISCORD_TOKEN`                 | ✓    | -                | Discord Bot トークン                              |
| `DATABASE_URL`                  | ✓    | -                | 例: `postgres://bot:pass@postgres:5432/bot`       |
| `RUST_LOG`                      |      | `info`           | `tracing-subscriber` のフィルタ                  |
| `RUST_LOG_FORMAT`               |      | `json`           | `pretty` で人間可読                              |
| `BOT_SCHEDULER_INTERVAL_SECS`   |      | `60`             | 放置検知の起動間隔                                |
| `BOT_SCHEDULER_BATCH_SIZE`      |      | `200`            | 1 ティックで処理する最大タスク数                   |
| `BOT_SHUTDOWN_TIMEOUT_SECS`     |      | `30`             | グレースフル停止の上限                            |
| `BOT_DEFAULT_LOCALE`            |      | `en-US`          | サーバー側でも未設定の場合のフォールバック        |

## 4. Discord 開発者ポータルでの設定

1. https://discord.com/developers/applications で「New Application」を作成。
2. **Bot** タブで Bot ユーザーを生成、トークンをコピー → `.env` に貼り付け。
3. **Privileged Gateway Intents** で `MESSAGE CONTENT INTENT` と `SERVER MEMBERS INTENT` は **不要**（フォーラムスレッドイベントのみで動きます）。`GUILD MEMBERS` のみ有効化を推奨。
4. **OAuth2 → URL Generator** で `bot` + `applications.commands` を選択。Permissions は最低限以下:
   - Send Messages
   - Manage Messages（アクティブタスクメッセージの編集／削除に必要）
   - Read Message History
   - Use Slash Commands
   - Mention Everyone（利用者ロールへのメンションに必要）
   - Create Public Threads
5. 生成 URL でサーバーに招待。

## 5. バックアップ

Postgres ボリューム（`docker volume ls` で確認できる `discord_forum_todo_postgres-data`）を `pg_dump` で定期取得してください。

```bash
docker exec -t discord_forum_todo-postgres-1 pg_dump -U bot -d bot > backup-$(date +%F).sql
```

復元:

```bash
docker exec -i discord_forum_todo-postgres-1 psql -U bot -d bot < backup-2026-05-14.sql
```

## 6. アップグレード

```bash
git pull
docker compose -f docker/docker-compose.yml pull
docker compose -f docker/docker-compose.yml up -d
```

マイグレーションは `migration` サービスが起動時に自動適用します。`up` が冪等なので何度実行しても安全です。

## 7. ログとメトリクス

- 標準出力に JSON ログが出ます。`docker compose logs -f bot` で確認。
- 構造化されているので Loki / Datadog / CloudWatch に転送する場合の取り回しが楽です。
- メトリクスは `tracing` イベントとして `target: "metrics"` で出ます。Prometheus 連携は将来対応予定。

## 8. トラブルシューティング

| 症状                                       | 対処                                                                 |
|--------------------------------------------|----------------------------------------------------------------------|
| Bot が起動しない                            | `docker compose logs bot` でエラーを確認。`DISCORD_TOKEN` が空でないか。 |
| `/config` がコマンド一覧に出ない             | Discord 側のコマンド登録が反映されていない可能性。再起動 + 数分待つ。 |
| アクティブタスクメッセージが更新されない    | Bot が当該チャンネルに「メッセージ管理」権限を持っているか確認。       |
| マイグレーションが失敗する                  | DB 接続情報を確認。`docker compose run migration up` を手動で実行して詳細ログを取得。|
| 大量の重複通知が来る                        | `notify-cooldown` を長くしてください。                              |

## 9. スケール

- 数百ギルド程度: 単一インスタンスで十分。
- 数千ギルド以上: 複数インスタンス可能。スケジューラは `FOR UPDATE SKIP LOCKED` で衝突しないように設計されています。Bot ゲートウェイ側はシャーディング（`twilight-gateway` の `ShardScheme`）で水平分割してください。
