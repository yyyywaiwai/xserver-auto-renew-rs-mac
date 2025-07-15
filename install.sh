#!/usr/bin/env bash
set -euo pipefail

# simple colors for a slightly fancier output
GREEN="\033[0;32m"
BOLD="\033[1m"
RESET="\033[0m"

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

echo -e "${BOLD}Installing xrenew ${version} for ${target}...${RESET}"

tmpdir=$(mktemp -d)
url="https://github.com/$repo/releases/download/$version/xrenew-${target}.tar.gz"
echo -e "${BOLD}Downloading${RESET} $url"
curl -sSfL "$url" | tar -xz -C "$tmpdir"

install -Dm755 "$tmpdir/xrenew" "$HOME/.local/bin/xrenew" || sudo install -Dm755 "$tmpdir/xrenew" /usr/local/bin/xrenew
rm -rf "$tmpdir"

hash -r 2>/dev/null || true

if [ -x "$HOME/.local/bin/xrenew" ]; then
  export PATH="$HOME/.local/bin:$PATH"
  "$HOME/.local/bin/xrenew" refresh || true
elif [ -x "/usr/local/bin/xrenew" ]; then
  "/usr/local/bin/xrenew" refresh || true
else
  echo "Warning: xrenew not found in expected locations" >&2
fi

printf "${GREEN}${BOLD}xrenew installed!${RESET}\n"
echo -e "まず ${BOLD}xrenew login${RESET}\n次に ${BOLD}xrenew enable${RESET} を実行してください。"
