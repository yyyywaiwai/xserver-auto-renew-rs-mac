# xrenew

**Xserver VPS の無料契約を自動で更新するコマンドラインツール**

`xrenew` は、Xserver VPS の無料契約を自動で延長するためのコマンドラインツールです。Rust 製で軽量に動作し、Linux サーバー上で簡単にセットアップできます。

---

## ✨ 主な特徴

- **🤖 自動契約更新**: systemd タイマーを利用して、定期的に契約を自動で延長します。
- **CAPTCHA 対応**: 画像認証に対応済みです。
- **🔔 Discord 通知**: 契約更新の結果を Discord の Webhook 経由で通知できます。
- **軽量動作**: Rust 製のため、リソースの消費が少なく、Xserver の無料 VPS 内でも快適に動作します。
- **簡単セットアップ**: インストールから自動化設定まで、数個のコマンドで完了します。

---

## 🖥️ 動作環境

### Linux
- **OS**: Linux (x86_64 / arm64)
- **必須コンポーネント**: `systemd`

### macOS
- **OS**: macOS (x86_64 / arm64)
- **必須コンポーネント**: Rust/Cargo (for building from source)

---

## 🚀 インストール

### Linux

以下のコマンドを 1 行実行するだけで、最新バージョンが自動でインストールされます。

```bash
curl -sSf https://raw.githubusercontent.com/h-sumiya/xserver-auto-renew-rs/main/install.sh | bash
```

### macOS

macOS では、以下の手順でソースからビルドしてインストールします。

1. Rust がインストールされていない場合、先にインストール:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. リポジトリをクローンしてインストール:
```bash
git clone https://github.com/yyyywaiwai/xserver-auto-renew-rs.git
cd xserver-auto-renew-rs
./install.sh
```

---

## 🏁 セットアップ手順

インストール後、以下の手順でセットアップを行ってください。

### 1\. ログイン

まず、`login` コマンドを実行して Xserver アカウント情報を登録します。プロンプトに従ってメールアドレスとパスワードを入力してください。

```bash
xrenew login
```

> [\!IMPORTANT]
> 初回ログイン時には **二段階認証** を求められる場合があります。必ずこの`login`コマンドを最初に実行し、認証を済ませてください。

### 2\. 自動延長の有効化

次に、`enable` コマンドを実行して、契約の自動延長を有効化します。これにより、12 時間ごとに自動で延長処理が実行されるようになります。

```bash
xrenew enable
```

**注意**: Linux では systemd、macOS では launchd を使用して自動実行を管理します。

### 3\. (オプション) Discord 通知設定

更新結果を Discord で受け取りたい場合は、以下のコマンドで Webhook URL を設定してください。

```bash
xrenew webhook <YOUR_DISCORD_WEBHOOK_URL>
```

---

## 🛠️ コマンド一覧

| コマンド               | 説明                                                                     |
| ---------------------- | ------------------------------------------------------------------------ |
| `xrenew login`         | Xserver アカウントでログインし、認証情報を保存します。                   |
| `xrenew extend`        | 手動で契約を 1 回延長します。                                            |
| `xrenew enable`        | systemd タイマーを登録し、契約の自動延長を有効化します。                 |
| `xrenew disable`       | 自動延長のタイマーを無効化します。                                       |
| `xrenew status`        | アカウント情報、Webhook 設定、タイマーの状態、実行ログなどを表示します。 |
| `xrenew captcha <KEY>` | TwoCaptcha の API キーを設定します。                                     |
| `xrenew webhook <URL>` | 実行結果を通知する Discord Webhook URL を設定・更新します。              |
| `xrenew update`        | `xrenew`を最新バージョンにアップデートします。                           |
| `xrenew clear`         | 保存されているアカウント情報やログなど、すべてのデータを削除します。     |

---

### 免責事項

- 本ツールは Xserver の公式ツールではありません。ツールの利用はすべて **自己責任** でお願いします。
- 保存された認証情報は `~/.local/share/xrenew` 以下に保存されます。
