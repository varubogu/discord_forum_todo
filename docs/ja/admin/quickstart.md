# Discord サーバー管理者向けクイックスタート

> 対象: Bot を導入した Discord サーバーの管理者（サーバー所有者または「サーバー管理」権限保有者）。
> 所要時間: 約 5 分（既に Bot がサーバーに追加済みの場合）。

## このガイドのゴール

- 利用者ロールを Bot に登録する。
- プロジェクト（フォーラムチャンネル）、アクティブタスク表示先、通知チャンネル、監視カテゴリを設定する。
- 放置検知の閾値を決める。
- ロケールを選ぶ。

## 0. 事前準備

- Bot が既にサーバーに追加されている（運用者から招待 URL を受け取ってください）。
- Discord サーバーで「サーバー管理」権限を持っている。
- 用意しておくもの:
  - **利用者ロール**: タスクを消化する人たちに付与する Discord ロール（既存のものでも、専用に作成しても良い）。
  - **プロジェクト用フォーラムチャンネル**: 既存のフォーラムを使うか、新規作成する。
  - **アクティブタスク表示用テキストチャンネル**: Bot 専用。`@everyone` の「メッセージ送信」を不可、Bot ロールには「メッセージ送信」「メッセージ管理」「メッセージ履歴閲覧」を許可。
  - **通知用テキストチャンネル**: 放置タスク通知が来る場所。
  - **監視カテゴリ**（任意）: チャンネル名の自動カウントやタスク自動生成のトリガー対象。

## 1. 設定の流れ

最も簡単なのは対話ウィザード `/setup` ですが、ここでは個別コマンド版を示します。

```
/config set role role: @利用者
/config set forum channel: #プロジェクトA
/config set active-channel channel: #アクティブタスク
/config set notify-channel channel: #タスク通知
/config set watch-category category: 進行中チケット
/config set idle-threshold state: not_started days: 3
/config set idle-threshold state: in_progress days: 5
/config set notify-cooldown hours: 24
/config set locale value: ja-JP
```

各コマンドは管理者権限を持っている人のみ実行できます。

## 2. パターン設定（任意）

監視カテゴリの中で名前にパターンが一致するチャンネルを「カウント対象」または「自動タスク化トリガー」にできます。

```
/config set pattern kind: counter name: ticket pattern: ^ticket-.*
/config set pattern kind: counter name: closed pattern: ^closed-.*
/config set pattern kind: counter name: temp pattern: ^temp-.*
/config set pattern kind: trigger name: ticket pattern: ^ticket-.*
```

- `counter` … `/count channels pattern: ticket` でカウントできます。
- `trigger` … 監視カテゴリ内にパターンに合うチャンネルが新規作成されると、自動的にフォーラムにタスクスレッドが立ちます。

## 3. 現状確認

```
/config show
```

未設定項目は赤で強調表示されます。すべて緑になれば運用開始可能です。

## 4. 動作確認

1. プロジェクト用フォーラムに新規スレッドを 1 つ作成。
2. 数秒後、アクティブタスク表示用チャンネルの Bot メッセージが更新されることを確認。
3. スレッドの状態タグを「着手中」相当に変更（タグ運用は管理者ガイドの後半を参照）。
4. アクティブタスクメッセージが再度更新されることを確認。

## 5. 利用者へのアナウンス

利用者向けのマニュアル: [docs/ja/user/how-to-use.md](../user/how-to-use.md)
利用者ロールを付与した人にこのリンクを共有してください。

## 6. 困ったとき

- 設定を変えたい → `/config set <key>` で個別変更、`/config reset <key>` で個別解除。
- 利用者ロールを変更したい → `/config set role` でやり直し。
- 完全リセットしたい → 運用者に DB レベルでの削除を依頼してください。
- Bot が反応しない → [docs/ja/operator/deploy-docker.md](../operator/deploy-docker.md) の「トラブルシューティング」節を参照。

## 7. 権限まとめ

| ロール             | 主な操作                                        |
|--------------------|--------------------------------------------------|
| サーバー管理者     | `/config`、`/setup`、`/count`、`/task`           |
| 利用者ロール       | `/task list`、`/task transition`、`/task assign` |
| その他のメンバー   | Bot は無視（コマンドを叩いてもエラー応答）       |
