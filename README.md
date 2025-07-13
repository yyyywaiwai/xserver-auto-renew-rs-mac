# xrenew

xrenew は Xserver VPS の無料契約を自動で延長するためのコマンドラインツールです。Rust 製で軽量に動作します。

## インストール

以下のコマンドを実行すると最新バージョンのバイナリをダウンロードしてインストールできます。

```bash
curl -sSf https://raw.githubusercontent.com/h-sumiya/xserver-auto-renew-rs/main/install.sh | bash
```

## 使い方

1. `xrenew login`
   - 初回実行時はメールアドレスとパスワードの入力を求められます。
   - 二段階認証を有効にしている場合、認証コードの入力が必要です。
2. `xrenew extend`
   - 手動で延長処理を行います。保存済みの認証情報を利用して自動でログインします。
3. `xrenew enable`
   - systemd タイマーを登録し、毎日自動で延長処理を行います。
4. `xrenew disable`
   - 上記タイマーを無効化します。
5. `xrenew status`
   - 保存されているアカウント情報と実行ログを表示します。

### 動作環境

- Linux (x86_64 / arm64)
- systemd が動作する環境

### 注意事項

- 本ツールは Xserver 公式のものではありません。利用は自己責任でお願いします。
- 保存された認証情報は `~/.local/share/xrenew` 以下に保存されます。

