"""
cargo_tag.py - Cargo.tomlのversionからgitタグ(v{version})を付与してリモートへプッシュするスクリプト

使い方:
    python cargo_tag.py --path . --remote origin

前提:
    - gitがインストール済みで、作業ディレクトリがgitリポジトリであること
    - Python3.11以降では標準のtomllibを使用
      それ以前のバージョンでは `pip install toml` が必要
"""

from __future__ import annotations

import argparse
import pathlib
import subprocess
import sys

# Python 3.11 以降では tomllib を標準で利用
try:
    import tomllib as toml  # type: ignore
except ModuleNotFoundError:  # Python 3.10 以前
    import toml  # type: ignore


def run(cmd: list[str]) -> str:
    """シェルコマンドを実行し、stdout を返す。失敗時は終了。"""
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        sys.exit(result.returncode)
    return result.stdout.strip()


def get_version(cargo_toml: pathlib.Path) -> str:
    """Cargo.toml から package.version を取得"""
    data = toml.load(cargo_toml.open("rb"))
    try:
        return data["package"]["version"]
    except KeyError:
        sys.stderr.write("Cargo.toml に package.version が見つかりません。\n")
        sys.exit(1)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Cargo.tomlのバージョンをGitタグにしてプッシュする",
    )
    parser.add_argument(
        "--path",
        default=".",
        help="Cargo.toml が置かれたディレクトリ (default: 現在のディレクトリ)",
    )
    parser.add_argument(
        "--remote",
        default="origin",
        help="タグをプッシュするリモート名 (default: origin)",
    )
    args = parser.parse_args()

    cargo_toml = pathlib.Path(args.path).expanduser().resolve() / "Cargo.toml"
    if not cargo_toml.is_file():
        sys.stderr.write(f"{cargo_toml} が見つかりません。\n")
        sys.exit(1)

    version = get_version(cargo_toml)
    tag_name = f"v{version}"

    # 既存タグの確認
    existing_tags = run(["git", "tag"]).splitlines()
    if tag_name in existing_tags:
        print(f"既にタグ {tag_name} が存在します。上書きせずにプッシュします。")
    else:
        run(["git", "tag", tag_name])
        print(f"タグ {tag_name} を作成しました。")

    # リモートへタグをプッシュ
    run(["git", "push", args.remote, tag_name])
    print(f"タグ {tag_name} を {args.remote} にプッシュしました。")


if __name__ == "__main__":
    main()
