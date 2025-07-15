# xrenew

xrenew は Xserver VPS の無料契約を自動で延長するためのコマンドラインツールです。Rust 製で軽量に動作します。

> [!NOTE]
> Xserver の無料 VPS 内で実行することも可能です。

## インストール

以下のコマンドを実行すると最新バージョンのバイナリをダウンロードしてインストールできます。

```bash
curl -sSf https://raw.githubusercontent.com/h-sumiya/xserver-auto-renew-rs/main/install.sh | bash
```

## 使い方

1. `xrenew login`
   - 初回実行時はメールアドレスとパスワードの入力を求められます。
2. `xrenew extend`
   - 手動で延長処理を行います。保存済みの認証情報を利用して自動でログインします。
3. `xrenew enable`
   - systemd タイマーを登録し、12時間おきに自動で延長処理を試みます。
4. `xrenew webhook <URL>`
   - 実行結果を Discord へ通知する Webhook URL を設定します。
5. `xrenew disable`
   - 上記タイマーを無効化します。
6. `xrenew status`
   - 保存されているアカウント情報と実行ログ、Webhook設定、タイマー状態を表示します。
7. `xrenew update`
   - 最新バージョンが公開されている場合自動でアップデートします。

> [!IMPORTANT]
> 初回実行時に二段階認証が求められる場合があります。
> そのため必ずインストール後 `enable`を実行前に `login` を実行してください。

### 動作環境

- Linux (x86_64 / arm64)
- systemd が動作する環境

### 注意事項

- 本ツールは Xserver 公式のものではありません。利用は自己責任でお願いします。
- 保存された認証情報は `~/.local/share/xrenew` 以下に保存されます。
