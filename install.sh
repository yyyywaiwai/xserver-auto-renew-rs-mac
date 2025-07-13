#!/usr/bin/env bash
set -euo pipefail

repo="h-sumiya/xserver-auto-renew-rs"
version="${VERSION:-latest}"

if [ "$(uname)" != "Linux" ]; then
  echo "This installer currently supports Linux only" >&2
  exit 1
fi

arch=$(uname -m)
case "$arch" in
  x86_64) target="x86_64-unknown-linux-gnu" ;;
  aarch64|arm64) target="aarch64-unknown-linux-gnu" ;;
  *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
esac

if [ "$version" = "latest" ]; then
  version=$(curl -sSfL "https://api.github.com/repos/$repo/releases/latest" | grep -Po '"tag_name":\s*"\K[^"]+')
fi

tmpdir=$(mktemp -d)
url="https://github.com/$repo/releases/download/$version/xrenew-${target}.tar.gz"
echo "Downloading $url"
curl -sSfL "$url" | tar -xz -C "$tmpdir"

install -Dm755 "$tmpdir/xrenew" "$HOME/.local/bin/xrenew" || sudo install -Dm755 "$tmpdir/xrenew" /usr/local/bin/xrenew
rm -rf "$tmpdir"

echo "xrenew installed! まず 'xrenew login' \n次に 'xrenew {定期実行を有効化}' を実行してください。"
